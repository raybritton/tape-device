use crate::constants::hardware::*;
use anyhow::{Error, Result};
use serde::Serialize;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub enum Param {
    Number(u8),
    DataReg(u8),
    AddrReg(u8),
    Addr(u16),
    Label(String),
    StrKey(String),
    DataKey(String),
}

bitflags! {
    pub(super) struct Parameters: u32 {
        const NUMBER =    0b00000001;
        const ADDRESS =   0b00000010;
        const DATA_REG =  0b00000100;
        const ADDR_REG =  0b00001000;
        const LABEL =     0b00010000;
        const STRING_KEY =0b00100000;
        const DATA_KEY =  0b01000000;
        const ADDRESSES = Self::LABEL.bits | Self::ADDRESS.bits;
        const REGISTERS = Self::DATA_REG.bits | Self::ADDR_REG.bits;
    }
}

impl Display for Parameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Parameters::NUMBER => write!(f, "byte"),
            Parameters::ADDRESS => write!(f, "address"),
            Parameters::DATA_REG => write!(f, "data_reg"),
            Parameters::DATA_KEY => write!(f, "data_key"),
            Parameters::ADDR_REG => write!(f, "addr_reg"),
            Parameters::LABEL => write!(f, "label"),
            Parameters::STRING_KEY => write!(f, "text_key"),
            Parameters::ADDRESSES => write!(f, "(label|address)"),
            Parameters::REGISTERS => write!(f, "(data_reg|addr_reg)"),
            _ => write!(f, ""),
        }
    }
}

impl Parameters {
    pub(super) fn parse(&self, input: &str) -> Result<Param> {
        let input = strip_trailing_comment(input);
        match *self {
            Parameters::NUMBER => parse_number(input),
            Parameters::DATA_REG => parse_data_reg(input),
            Parameters::ADDR_REG => parse_addr_reg(input),
            Parameters::ADDRESS => parse_addr(input),
            Parameters::LABEL => Ok(Param::Label(input.to_string())),
            Parameters::DATA_KEY => Ok(Param::DataKey(input.to_string())),
            Parameters::STRING_KEY => Ok(Param::StrKey(input.to_string())),
            Parameters::REGISTERS => {
                let data = parse_data_reg(input);
                let addr = parse_addr_reg(input);
                if data.is_ok() {
                    return data;
                }
                if addr.is_ok() {
                    return addr;
                }
                Err(Error::msg(format!(
                    "Expected data or addr reg, found {}",
                    input
                )))
            }
            Parameters::ADDRESSES => {
                if let Ok(addr) = parse_addr(input) {
                    Ok(addr)
                } else {
                    Ok(Param::Label(input.to_string()))
                }
            }
            _ => panic!("Unhandled param: {:?}", self),
        }
    }
}

fn strip_trailing_comment(input: &str) -> &str {
    let parts = input.split('#').collect::<Vec<&str>>();
    parts[0].trim()
}

fn parse_data_reg(input: &str) -> Result<Param> {
    match input.to_ascii_lowercase().as_str() {
        "d0" => Ok(Param::DataReg(REG_D0)),
        "d1" => Ok(Param::DataReg(REG_D1)),
        "d2" => Ok(Param::DataReg(REG_D2)),
        "d3" => Ok(Param::DataReg(REG_D3)),
        "acc" => Ok(Param::DataReg(REG_ACC)),
        _ => Err(Error::msg(format!("Not a valid data register: {}", input))),
    }
}

fn parse_addr_reg(input: &str) -> Result<Param> {
    match input.to_ascii_lowercase().as_str() {
        "a0" => Ok(Param::AddrReg(REG_A0)),
        "a1" => Ok(Param::AddrReg(REG_A1)),
        _ => Err(Error::msg(format!(
            "Not a valid address register: {}",
            input
        ))),
    }
}

fn parse_number(input: &str) -> Result<Param> {
    let num = if input.starts_with('x') {
        let hex = input.chars().skip(1).collect::<String>();
        u8::from_str_radix(&hex, 16)
    } else if input.starts_with('b') {
        if input.len() == 9 {
            let bin = input.chars().skip(1).collect::<String>();
            u8::from_str_radix(&bin, 2)
        } else {
            return Err(Error::msg(format!(
                "Error parsing number {}: must be b followed all 8 bits",
                input
            )));
        }
    } else if input.len() == 3 && input.starts_with('\'') && input.ends_with('\'') {
        let chr = input.chars().nth(1).unwrap();
        if chr.is_ascii() {
            Ok(chr as u8)
        } else {
            return Err(Error::msg(format!("Invalid char: {}", input)));
        }
    } else {
        input.parse::<u8>()
    };
    match num {
        Ok(num) => Ok(Param::Number(num)),
        Err(err) => Err(Error::msg(format!(
            "Error parsing number {}: {}",
            input, err
        ))),
    }
}

fn parse_addr(input: &str) -> Result<Param> {
    if !input.starts_with('@') {
        return Err(Error::msg("Address must start with @"));
    }
    let input = input.chars().skip(1).collect::<String>();
    let num = if input.starts_with('x') {
        let hex = input.chars().skip(1).collect::<String>();
        u16::from_str_radix(&hex, 16)
    } else {
        input.parse::<u16>()
    };
    match num {
        Ok(num) => Ok(Param::Addr(num)),
        Err(err) => Err(Error::msg(format!(
            "Error parsing number {}: {}",
            input, err
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_parsing() {
        assert_eq!(parse_addr("@10").unwrap(), Param::Addr(10));
        assert_eq!(parse_addr("@0").unwrap(), Param::Addr(0));
        assert_eq!(parse_addr("@100").unwrap(), Param::Addr(100));
        assert_eq!(parse_addr("@255").unwrap(), Param::Addr(255));
        assert_eq!(parse_addr("@1000").unwrap(), Param::Addr(1000));
        assert_eq!(parse_addr("@12000").unwrap(), Param::Addr(12000));
        assert_eq!(parse_addr("@65535").unwrap(), Param::Addr(65535));
        assert!(parse_addr("@65536").is_err());
        assert!(parse_addr("@-10").is_err());
        assert!(parse_addr("@test").is_err());
        assert_eq!(parse_addr("@255").unwrap(), Param::Addr(255));
        assert_eq!(parse_addr("@xA").unwrap(), Param::Addr(10));
        assert_eq!(parse_addr("@x0").unwrap(), Param::Addr(0));
        assert_eq!(parse_addr("@x64").unwrap(), Param::Addr(100));
        assert_eq!(parse_addr("@xFF").unwrap(), Param::Addr(255));
        assert_eq!(parse_addr("@xFF00").unwrap(), Param::Addr(65280));
        assert_eq!(parse_addr("@xFFFF").unwrap(), Param::Addr(65535));
        assert!(parse_addr("@x1FFFF").is_err());
        assert!(parse_addr("@1x2").is_err());
        assert!(parse_addr("a0").is_err());
        assert!(parse_addr("@x2p").is_err());
        assert!(parse_addr("@x").is_err());
    }

    #[test]
    fn test_number_parsing() {
        assert_eq!(parse_number("10").unwrap(), Param::Number(10));
        assert_eq!(parse_number("0").unwrap(), Param::Number(0));
        assert_eq!(parse_number("100").unwrap(), Param::Number(100));
        assert_eq!(parse_number("255").unwrap(), Param::Number(255));
        assert!(parse_number("256").is_err());
        assert!(parse_number("1000").is_err());
        assert!(parse_number("-1").is_err());
        assert_eq!(parse_number("xA").unwrap(), Param::Number(10));
        assert_eq!(parse_number("x0").unwrap(), Param::Number(0));
        assert_eq!(parse_number("x64").unwrap(), Param::Number(100));
        assert_eq!(parse_number("xFF").unwrap(), Param::Number(255));
        assert!(parse_number("x100").is_err());
        assert!(parse_number("x3e8").is_err());
        assert!(parse_number("xF001").is_err());
        assert_eq!(parse_number("' '").unwrap(), Param::Number(32));
        assert_eq!(parse_number("'s'").unwrap(), Param::Number(115));
        assert_eq!(parse_number("'}'").unwrap(), Param::Number(125));
        assert!(parse_number("'s '").is_err());
        assert!(parse_number("''").is_err());
        assert_eq!(parse_number("b00000000").unwrap(), Param::Number(0));
        assert_eq!(parse_number("b11111111").unwrap(), Param::Number(255));
        assert_eq!(parse_number("b00001111").unwrap(), Param::Number(15));
        assert!(parse_number("b0").is_err());
        assert!(parse_number("b1").is_err());
        assert!(parse_number("b101010100111").is_err());
    }

    #[test]
    fn test_reg_parsing() {
        assert_eq!(parse_data_reg("d0").unwrap(), Param::DataReg(REG_D0));
        assert_eq!(parse_data_reg("d1").unwrap(), Param::DataReg(REG_D1));
        assert_eq!(parse_data_reg("d2").unwrap(), Param::DataReg(REG_D2));
        assert_eq!(parse_data_reg("d3").unwrap(), Param::DataReg(REG_D3));
        assert_eq!(parse_data_reg("acc").unwrap(), Param::DataReg(REG_ACC));
        assert_eq!(parse_addr_reg("a0").unwrap(), Param::AddrReg(REG_A0));
        assert_eq!(parse_addr_reg("a1").unwrap(), Param::AddrReg(REG_A1));
        assert!(parse_data_reg("d5").is_err());
        assert!(parse_data_reg("a0").is_err());
        assert!(parse_data_reg("").is_err());
        assert!(parse_addr_reg("").is_err());
        assert!(parse_data_reg("dec").is_err());
        assert!(parse_addr_reg("d0").is_err());
        assert!(parse_addr_reg("acc").is_err());
    }

    #[test]
    fn test_number_parameter_parsing() {
        assert_eq!(Parameters::NUMBER.parse("10").unwrap(), Param::Number(10));
    }

    #[test]
    fn test_addr_reg_parameter_parsing() {
        assert_eq!(
            Parameters::ADDR_REG.parse("a0").unwrap(),
            Param::AddrReg(REG_A0)
        );
    }

    #[test]
    fn test_addresses_parameter_parsing() {
        assert_eq!(Parameters::ADDRESSES.parse("@34").unwrap(), Param::Addr(34));
    }

    #[test]
    fn test_addr_parameter_parsing() {
        assert_eq!(Parameters::ADDRESS.parse("@986").unwrap(), Param::Addr(986));
    }

    #[test]
    fn test_data_reg_parameter_parsing() {
        assert_eq!(
            Parameters::DATA_REG.parse("D0").unwrap(),
            Param::DataReg(REG_D0)
        );
    }

    #[test]
    fn test_label_parameter_parsing() {
        assert_eq!(
            Parameters::LABEL.parse("start").unwrap(),
            Param::Label(String::from("start"))
        );
    }

    #[test]
    fn test_data_key_parameter_parsing() {
        assert_eq!(
            Parameters::DATA_KEY.parse("start").unwrap(),
            Param::DataKey(String::from("start"))
        );
    }

    #[test]
    fn test_string_key_parameter_parsing() {
        assert_eq!(
            Parameters::STRING_KEY.parse("greeting").unwrap(),
            Param::StrKey(String::from("greeting"))
        );
    }

    #[test]
    fn test_registers_parameter_parsing() {
        assert_eq!(
            Parameters::REGISTERS.parse("aCc").unwrap(),
            Param::DataReg(REG_ACC)
        );
    }
}
