use serde::Serialize;

#[derive(Serialize, Debug, Default)]
pub struct DebugModel {
    pub ops: Vec<DebugOp>,
    pub strings: Vec<DebugDataString>,
    pub data: Vec<DebugDataString>,
    pub labels: Vec<DebugLabel>,
}

#[derive(Serialize, Debug)]
pub struct DebugOp {
    byte: usize,
    original_line: String,
    line_num: usize,
    processed_line: String,
}

#[derive(Serialize, Debug)]
pub struct DebugDataString {
    addr: usize,
    key: String,
    original_line: String,
    line_num: usize,
    usage: Vec<usize>,
}

#[derive(Serialize, Debug)]
pub struct DebugLabel {
    byte: usize,
    name: String,
    original_line: String,
    line_num: usize,
    usage: Vec<usize>,
}

impl DebugOp {
    pub fn new(
        byte: usize,
        original_line: String,
        line_num: usize,
        processed_line: String,
    ) -> Self {
        DebugOp {
            byte,
            original_line,
            line_num,
            processed_line,
        }
    }
}

impl DebugDataString {
    pub fn new(addr: usize, key: String, original_line: String, line_num: usize) -> Self {
        DebugDataString {
            addr,
            key,
            original_line,
            line_num,
            usage: vec![],
        }
    }
}

impl DebugLabel {
    pub fn new(byte: usize, name: String, original_line: String, line_num: usize) -> Self {
        DebugLabel {
            byte,
            name,
            original_line,
            line_num,
            usage: vec![],
        }
    }
}