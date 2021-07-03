use crate::assembler::program_model::{
    AddressReplacement, DataModel, LabelModel, OpModel, ProgramModel, StringModel,
};
use crate::constants::get_addr_byte_offset;
use crate::constants::hardware::{MAX_DATA_BYTES, MAX_STRING_BYTES};
use crate::constants::system::{PRG_VERSION, TAPE_HEADER_1, TAPE_HEADER_2};
use anyhow::{Error, Result};
use std::collections::{BTreeMap, HashMap};

pub fn generate_byte_code(mut program_model: ProgramModel) -> Result<Vec<u8>> {
    let mut output = vec![TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION];
    output.push(program_model.name.len() as u8);
    output.extend_from_slice(program_model.name.as_bytes());
    output.push(program_model.version.len() as u8);
    output.extend_from_slice(program_model.version.as_bytes());

    let ops_output =
        generate_ops_bytes(&program_model.ops, output.len() + 2, program_model.labels)?; //+2 for op byte count written after len is known

    output.extend_from_slice(&(ops_output.bytes.len() as u16).to_be_bytes());
    output.extend_from_slice(&ops_output.bytes);

    let (string_bytes, string_addresses) = generate_string_bytes(program_model.strings)?;

    output.extend_from_slice(&(string_bytes.len() as u16).to_be_bytes());
    output.extend_from_slice(&string_bytes);

    let (data_bytes, data_addresses) = generate_data_bytes(program_model.data)?;

    output.extend_from_slice(&data_bytes);

    output = update_addresses(output, ops_output.string_targets, string_addresses);
    output = update_addresses(output, ops_output.data_targets, data_addresses);
    output = update_addresses(output, ops_output.label_targets, ops_output.label_addresses);

    Ok(output)
}

fn update_addresses(
    mut bytes: Vec<u8>,
    targets: HashMap<String, Vec<u16>>,
    sources: HashMap<String, u16>,
) -> Vec<u8> {
    for (key, source) in sources {
        if let Some(op_offsets) = targets.get(&key) {
            for offset in op_offsets {
                let addr = source.to_be_bytes();
                bytes[*offset as usize] = addr[0];
                bytes[(*offset + 1) as usize] = addr[1];
            }
        }
    }
    bytes
}

#[derive(Debug, Default)]
struct OpsOutput {
    bytes: Vec<u8>,
    string_targets: HashMap<String, Vec<u16>>,
    data_targets: HashMap<String, Vec<u16>>,
    label_targets: HashMap<String, Vec<u16>>,
    label_addresses: HashMap<String, u16>,
}

fn generate_ops_bytes(
    ops: &[OpModel],
    offset: usize,
    labels: HashMap<String, LabelModel>,
) -> Result<OpsOutput> {
    let mut labels: BTreeMap<usize, LabelModel> = convert_label_map_to_linenum(labels);
    let mut output = OpsOutput::default();
    for op in ops {
        if !labels.is_empty() {
            let lbl_line_num = labels
                .values()
                .next()
                .unwrap()
                .definition
                .as_ref()
                .unwrap()
                .line_num;
            if lbl_line_num <= op.line_num {
                output.label_addresses.insert(
                    labels.values().next().unwrap().key.clone(),
                    output.bytes.len() as u16,
                );
                labels.remove(&lbl_line_num);
            }
        }
        let (bytes, replacement) = op.to_bytes();
        if replacement != AddressReplacement::None {
            let (key, targets) = match replacement {
                AddressReplacement::None => panic!("System error: None after a not none check"),
                AddressReplacement::Label(key) => (key, &mut output.label_targets),
                AddressReplacement::Str(key) => (key, &mut output.string_targets),
                AddressReplacement::Data(key) => (key, &mut output.data_targets),
            };
            let param_offset = get_addr_byte_offset(op.opcode);
            targets
                .entry(key)
                .or_insert_with(Vec::new)
                .push((output.bytes.len() + param_offset + offset) as u16);
        }
        output.bytes.extend_from_slice(&bytes);
    }

    Ok(output)
}
fn generate_data_bytes(
    data: HashMap<String, DataModel>,
) -> Result<(Vec<u8>, HashMap<String, u16>)> {
    let mut output = vec![];
    let mut addresses = HashMap::new();
    let mut list: Vec<(String, DataModel)> = data.into_iter().collect();
    list.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    for (key, data_model) in list {
        if (output.len() + data_model.content.len()) > MAX_DATA_BYTES {
            return Err(Error::msg(format!(
                "Too much data at `{}` on line {}, max {} bytes but is at least {} bytes",
                data_model.definition.original_line,
                data_model.definition.line_num,
                MAX_DATA_BYTES,
                output.len() + data_model.content.len()
            )));
        }
        addresses.insert(key, output.len() as u16);
        output.extend_from_slice(&data_model.content);
    }

    Ok((output, addresses))
}

fn generate_string_bytes(
    strings: HashMap<String, StringModel>,
) -> Result<(Vec<u8>, HashMap<String, u16>)> {
    let mut output = vec![];
    let mut addresses = HashMap::new();
    let mut list: Vec<(String, StringModel)> = strings.into_iter().collect();
    list.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    for (key, string_model) in list {
        if (output.len() + string_model.content.len()) > MAX_STRING_BYTES {
            return Err(Error::msg(format!(
                "Too many strings at `{}` on line {}, max {} bytes but is at least {} bytes",
                string_model.definition.original_line,
                string_model.definition.line_num,
                MAX_STRING_BYTES,
                output.len() + string_model.content.len()
            )));
        }
        addresses.insert(key, output.len() as u16);
        output.push(string_model.content.len() as u8);
        output.extend_from_slice(string_model.content.as_bytes());
    }

    Ok((output, addresses))
}

fn get_label_keys_for_line(labels: &HashMap<String, LabelModel>, line_num: usize) -> Vec<String> {
    labels
        .values()
        .filter(|model| {
            model
                .definition
                .as_ref()
                .map(|def| def.line_num == line_num)
                .unwrap_or(false)
        })
        .map(|model| model.key.clone())
        .collect()
}

fn convert_label_map_to_linenum(
    labels: HashMap<String, LabelModel>,
) -> BTreeMap<usize, LabelModel> {
    labels
        .into_iter()
        .map(|(_, model)| (model.definition.as_ref().unwrap().line_num, model))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::constants::code::{
        ADD_REG_REG, CPY_REG_REG, INC_REG, LD_AREG_DATA_VAL_REG, PRTS_STR,
    };
    use crate::constants::hardware::*;
    use crate::language::parser::params::Param;

    #[test]
    #[rustfmt::skip]
    fn test_update_addresses_single() {
        let bytes = vec![PRTS_STR, 0, 0];
        let mut targets = HashMap::new();
        let mut sources = HashMap::new();
        
        targets.insert(String::from("foo"), vec![1]);
        sources.insert(String::from("abc"), 0);
        sources.insert(String::from("foo"), 4);
        
        let output = update_addresses(bytes, targets, sources);
        assert_eq!(output, vec![PRTS_STR, 0, 4]);
    }

    #[test]
    #[rustfmt::skip]
    fn test_gen_string_bytes() {
        let mut strings = HashMap::new();
        strings.insert(
            String::from("a"),
            StringModel::new(String::new(), String::from("test string"), String::new(), 0),
        );
        strings.insert(
            String::from("b"),
            StringModel::new(String::new(), String::from("an example"), String::new(), 0),
        );
        strings.insert(
            String::from("c"),
            StringModel::new(String::new(), String::from("abcdef"), String::new(), 0),
        );
        
        let (bytes, sources) = generate_string_bytes(strings).unwrap();
        let mut expected = HashMap::new();
        expected.insert(String::from("a"), 0_u16);
        expected.insert(String::from("b"), 12);
        expected.insert(String::from("c"), 23);
        
        assert_eq!(bytes, vec![
            11, 116, 101, 115, 116, 32, 115, 116, 114, 105, 110, 103,
            10, 97, 110, 32, 101, 120, 97, 109, 112, 108, 101,
            6, 97, 98, 99, 100, 101, 102
        ]);
        
        assert_eq!(expected.len(), sources.len());
        assert_eq!(expected.get("a"), sources.get("a"));
        assert_eq!(expected.get("b"), sources.get("b"));
        assert_eq!(expected.get("c"), sources.get("c"));
    }

    #[test]
    #[rustfmt::skip]
    fn test_gen_data_bytes() {
        let mut data = HashMap::new();
        data.insert(
            String::from("a"),
            DataModel::new(String::new(), vec![2,5,2,1,2,3,4,5,6,7], String::new(), 0),
        );
        data.insert(
            String::from("b"),
            DataModel::new(String::new(), vec![4,2,2,2,2,97,98,99,100,101,102,103,104], String::new(), 0),
        );

        let (bytes, sources) = generate_data_bytes(data).unwrap();
        let mut expected = HashMap::new();
        expected.insert(String::from("a"), 0_u16);
        expected.insert(String::from("b"), 10);

        assert_eq!(bytes, vec![
            2,5,2,1,2,3,4,5,6,7,
            4,2,2,2,2,97,98,99,100,101,102,103,104
        ]);

        assert_eq!(expected.len(), sources.len());
        assert_eq!(expected.get("a"), sources.get("a"));
        assert_eq!(expected.get("b"), sources.get("b"));
    }

    mod generate_ops {
        use super::*;
        use crate::constants::code::LD_AREG_DATA_REG_VAL;
        use crate::language::parser::params::Param::*;

        #[test]
        #[rustfmt::skip]
        fn test_target_gen() {
            let output = generate_ops_bytes(
                &[
                    OpModel::new(PRTS_STR, vec![StrKey(String::from("foo"))], String::new(), String::from("prts foo"), 0),
                    OpModel::new(LD_AREG_DATA_REG_VAL, vec![AddrReg(REG_A0), DataKey(String::from("bar")), DataReg(REG_D2), Number(10), ], String::new(), String::from("ld a0 bar d2 10"), 0),
                ],
                0,
                HashMap::new()
            )
            .unwrap();

            let mut expected_strings = HashMap::new();
            expected_strings.insert(String::from("foo"), vec![1_u16]);
            let mut expected_data = HashMap::new();
            expected_data.insert(String::from("bar"), vec![5_u16]);

            assert_eq!(
                output.bytes,
                vec![
                    PRTS_STR, 0, 0,
                    LD_AREG_DATA_REG_VAL, REG_A0, 0, 0, REG_D2, 10
                ]
            );
            assert_eq!(output.string_targets, expected_strings);
            assert_eq!(output.data_targets, expected_data);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_header() {
        let model = ProgramModel::new(String::from("Test Prog"), String::from("1.0"));
        let bytes = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                9, 84, 101, 115, 116, 32, 80, 114, 111, 103, 
                3, 49, 46, 48,
                0, 0,
                0, 0
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));
        
        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_D0)], String::new(), String::from("inc d0"), 0));
        model.ops.push(OpModel::new(CPY_REG_REG, vec![Param::DataReg(REG_D1), Param::DataReg(REG_D0)], String::new(), String::from("cpy d1 d0"), 1));
        model.ops.push(OpModel::new(ADD_REG_REG, vec![Param::DataReg(REG_D0), Param::DataReg(REG_D1)], String::new(), String::from("add d0 d1"), 2));
        
        let bytes = generate_byte_code(model).unwrap();
        
        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 8,
                INC_REG, REG_D0,
                CPY_REG_REG, REG_D1, REG_D0,
                ADD_REG_REG, REG_D0, REG_D1,
                0, 0
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog_with_strings() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.strings.insert(String::from("abc"), StringModel::new(String::from("abc"), String::from("foo"), String::new(), 0));
        model.strings.insert(String::from("test"), StringModel::new(String::from("test"), String::from("bar"), String::new(), 0));
        
        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_D0)], String::new(), String::from("inc d0"), 0));
        model.ops.push(OpModel::new(PRTS_STR, vec![Param::StrKey(String::from("test"))], String::new(), String::from("prts test"), 1));

        let bytes = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 5,
                INC_REG, REG_D0,
                PRTS_STR, 0, 4,
                0, 8,
                3, 102, 111, 111,
                3, 98, 97, 114
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog_with_data() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.data.insert(String::from("dk1"), DataModel::new(String::new(), vec![3,2,2,4,10,11,50,51,97,98,99,100], String::new(), 0));
        model.data.insert(String::from("dk2"), DataModel::new(String::new(), vec![1,10,30,31,32,33,34,35,36,37,38,39], String::new(), 0));

        model.ops.push(OpModel::new(ADD_REG_REG, vec![Param::DataReg(REG_D0), Param::DataReg(REG_D1)], String::new(), String::from("add d0 d1"), 0));
        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_ACC)], String::new(), String::from("inc acc"), 0));
        model.ops.push(OpModel::new(LD_AREG_DATA_VAL_REG, vec![Param::AddrReg(REG_A0), Param::DataKey(String::from("dk2")), Param::Number(2), Param::DataReg(REG_D3)], String::new(), String::from("ld a0 dk1 2 d3"), 1));

        let bytes = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 11,
                ADD_REG_REG, REG_D0, REG_D1,
                INC_REG, REG_ACC,
                LD_AREG_DATA_VAL_REG, REG_A0, 0, 12, 2, REG_D3,
                0, 0,
                3,2,2,4,10,11,50,51,97,98,99,100,
                1,10,30,31,32,33,34,35,36,37,38,39
            ]
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_simple_prog_with_strings_and_data() {
        let mut model = ProgramModel::new(String::from("a"), String::from("b"));

        model.strings.insert(String::from("abc"), StringModel::new(String::from("abc"), String::from("foo"), String::new(), 0));
        model.data.insert(String::from("dk1"), DataModel::new(String::new(), vec![3,2,2,4,10,11,50,51,97,98,99,100], String::new(), 0));

        model.ops.push(OpModel::new(ADD_REG_REG, vec![Param::DataReg(REG_D0), Param::DataReg(REG_D1)], String::new(), String::from("add d0 d1"), 0));
        model.ops.push(OpModel::new(INC_REG, vec![Param::DataReg(REG_ACC)], String::new(), String::from("inc acc"), 0));
        model.ops.push(OpModel::new(LD_AREG_DATA_VAL_REG, vec![Param::AddrReg(REG_A0), Param::DataKey(String::from("dk1")), Param::Number(2), Param::DataReg(REG_D3)], String::new(), String::from("ld a0 dk1 2 d3"), 1));
        model.ops.push(OpModel::new(PRTS_STR, vec![Param::StrKey(String::from("abc"))], String::new(), String::from("prts abc"), 0));

        let bytes = generate_byte_code(model).unwrap();

        assert_eq!(
            bytes,
            vec![
                TAPE_HEADER_1, TAPE_HEADER_2, PRG_VERSION,
                1, 97,
                1, 98,
                0, 14,
                ADD_REG_REG, REG_D0, REG_D1,
                INC_REG, REG_ACC,
                LD_AREG_DATA_VAL_REG, REG_A0, 0, 0, 2, REG_D3,
                PRTS_STR, 0, 0,
                0, 4,
                3, 102, 111, 111,
                3,2,2,4,10,11,50,51,97,98,99,100
            ]
        )
    }
}