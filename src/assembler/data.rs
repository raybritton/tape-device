use crate::assembler::FORMAT_ERROR;
use anyhow::{Error, Result};
use std::collections::HashMap;

pub(super) fn compile_strings(
    lines: &mut Vec<String>,
    keep_whitespace: bool,
) -> Result<(HashMap<String, u16>, Vec<u8>)> {
    let mut mapping = HashMap::with_capacity(lines.len());
    let mut output = Vec::with_capacity(lines.len() * 10);
    let mut line = lines.remove(0);
    while line != ".ops" {
        if let Some(idx) = line.find('=') {
            let (key, content) = line.split_at(idx);
            let mut content: String = content.chars().skip(1).collect();
            if !keep_whitespace {
                content = content.trim().to_string();
            }
            let key = key.trim();
            if key
                .chars()
                .any(|chr| !(chr.is_ascii_alphanumeric() || chr == '_'))
            {
                return Err(Error::msg(format!(
                    "Line '{}' has invalid key must be [a-zA-Z0-9_]+",
                    line
                )));
            }
            if content.len() > 255 {
                return Err(Error::msg(format!("Line '{}' in strings is too long, must be at most 255 chars (including whitespace if --keep_whitespace)", line)));
            }
            if output.len() >= u16::MAX as usize {
                return Err(Error::msg(format!("Too many strings at '{}', max of {} chars in strings data including whitespace but not including keys", line, u16::MAX - 1)));
            }

            mapping.insert(key.to_string(), output.len() as u16);
            output.push(content.len() as u8);
            output.extend_from_slice(content.as_bytes());
        } else {
            return Err(Error::msg(format!(
                "Unexpected string definition: {}",
                line
            )));
        }
        if lines.is_empty() {
            return Err(Error::msg(format!(
                "Unexpected EoF while compiling strings, check ops section starts with .ops\n\n{}",
                FORMAT_ERROR
            )));
        }
        line = lines.remove(0);
    }

    output.shrink_to_fit();
    mapping.shrink_to_fit();
    Ok((mapping, output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_file() {
        let mut input = vec![
            String::from("simple=test"),
            String::from("checking=bytes"),
            String::from(".ops"),
        ];
        let result = compile_strings(&mut input, false);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        //Keys and values are sorted as otherwise they are in random order
        //So just checking the expected values are somewhere
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert!(keys.contains(&&String::from("simple")));
        assert!(keys.contains(&&String::from("checking")));
        assert!(values.contains(&&0));
        assert!(values.contains(&&5));
        assert_eq!(result.1, [4, 116, 101, 115, 116, 5, 98, 121, 116, 101, 115]);
    }

    #[test]
    fn test_whitespace() {
        let mut input = vec![
            String::from("no=whitespace"),
            String::from("some=  before"),
            String::from("and=after  "),
            String::from("also=  both  "),
            String::from(" keyb=ws"),
            String::from("keya =ws"),
            String::from(".ops"),
        ];
        let mut input2 = input.clone();
        let result_no = compile_strings(&mut input, false);
        let result_ws = compile_strings(&mut input2, true);
        assert!(result_no.is_ok());
        assert!(result_ws.is_ok());
        let result_no = result_no.unwrap();
        let result_ws = result_ws.unwrap();
        let keys_ws = result_ws.0.keys().collect::<Vec<&String>>();
        let keys_no = result_no.0.keys().collect::<Vec<&String>>();
        let bytes_ws = result_ws.1;
        let bytes_no = result_no.1;
        assert!(keys_no.contains(&&String::from("no")));
        assert!(keys_no.contains(&&String::from("some")));
        assert!(keys_no.contains(&&String::from("and")));
        assert!(keys_no.contains(&&String::from("also")));
        assert!(keys_no.contains(&&String::from("keya")));
        assert!(keys_no.contains(&&String::from("keyb")));
        assert!(keys_ws.contains(&&String::from("no")));
        assert!(keys_ws.contains(&&String::from("some")));
        assert!(keys_ws.contains(&&String::from("and")));
        assert!(keys_ws.contains(&&String::from("also")));
        assert!(keys_ws.contains(&&String::from("keya")));
        assert!(keys_ws.contains(&&String::from("keyb")));
        assert_eq!(
            bytes_no,
            [
                10, 119, 104, 105, 116, 101, 115, 112, 97, 99, 101, 6, 98, 101, 102, 111, 114, 101,
                5, 97, 102, 116, 101, 114, 4, 98, 111, 116, 104, 2, 119, 115, 2, 119, 115
            ]
        );
        assert_eq!(
            bytes_ws,
            [
                10, 119, 104, 105, 116, 101, 115, 112, 97, 99, 101, 8, 32, 32, 98, 101, 102, 111,
                114, 101, 7, 97, 102, 116, 101, 114, 32, 32, 8, 32, 32, 98, 111, 116, 104, 32, 32,
                2, 119, 115, 2, 119, 115
            ]
        );
    }

    #[test]
    fn test_output_order_matches() {
        let mut input = vec![
            String::from("simple=test"),
            String::from("checking=order"),
            String::from("of=output"),
            String::from(".ops"),
        ];
        let result = compile_strings(&mut input, false);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        let keys = result.0.keys().collect::<Vec<&String>>();
        let values = result.0.values().collect::<Vec<&u16>>();
        assert_eq!(keys.len(), values.len());
        for (idx, key) in keys.iter().enumerate() {
            let key = key.as_str();
            match key {
                "simple" => assert_eq!(values[idx], &0),
                "checking" => assert_eq!(values[idx], &5),
                "of" => assert_eq!(values[idx], &11),
                _ => assert!(false, "Invalid key {}", key),
            }
        }
    }

    #[test]
    fn test_ops_not_consumed() {
        let mut input = vec![
            String::from("a=string"),
            String::from(".ops"),
            String::from("INC D0"),
        ];
        let result = compile_strings(&mut input, false);
        assert_eq!(input.len(), 1);
        assert_eq!(input[0], String::from("INC D0"));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(
            result.0.keys().collect::<Vec<&String>>(),
            vec![&String::from("a")]
        );
        assert_eq!(result.0.values().collect::<Vec<&u16>>(), vec![&0]);
        assert_eq!(result.1, [6, 115, 116, 114, 105, 110, 103]);
    }

    #[test]
    fn test_just_ops_marker() {
        let mut input = vec![String::from(".ops")];
        let result = compile_strings(&mut input, false);
        assert!(input.is_empty());
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.0.is_empty());
        assert_eq!(result.1, vec![]);
    }

    #[test]
    fn test_no_ops_marker() {
        let mut input = vec![String::from("a=string")];
        assert!(compile_strings(&mut input, false).is_err());
        assert!(input.is_empty());
    }
}
