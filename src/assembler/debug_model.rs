use serde::Serialize;

#[derive(Serialize, Debug, Default, PartialEq)]
pub struct DebugModel {
    pub ops: Vec<DebugOp>,
    pub strings: Vec<DebugString>,
    pub data: Vec<DebugData>,
    pub labels: Vec<DebugLabel>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DebugOp {
    byte_addr: u16,
    original_line: String,
    line_num: usize,
    processed_line: String,
    bytes: Vec<u8>
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DebugString {
    addr: u16,
    pub(crate) key: String,
    content: String,
    original_line: String,
    line_num: usize,
    pub usage: Vec<DebugUsage>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DebugData {
    addr: u16,
    pub(crate) key: String,
    content: Vec<Vec<u8>>,
    original_line: String,
    line_num: usize,
    pub usage: Vec<DebugUsage>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DebugLabel {
    byte: u16,
    pub(crate) name: String,
    original_line: String,
    line_num: usize,
    pub usage: Vec<DebugUsage>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DebugUsage {
    op_addr: u16,
    offset: u8,
    line: usize
}

impl DebugModel {
    #[allow(dead_code)]
    pub fn new(
        ops: Vec<DebugOp>,
        strings: Vec<DebugString>,
        data: Vec<DebugData>,
        labels: Vec<DebugLabel>,
    ) -> Self {
        DebugModel {
            ops,
            strings,
            data,
            labels,
        }
    }
}

impl DebugOp {
    pub fn new(
        byte_addr: u16,
        original_line: String,
        line_num: usize,
        processed_line: String,
        bytes: Vec<u8>
    ) -> Self {
        DebugOp {
            byte_addr,
            original_line,
            line_num,
            processed_line,
            bytes
        }
    }
}

impl DebugString {
    pub fn new(addr: u16, key: String, content: String, original_line: String, line_num: usize) -> Self {
        DebugString {
            addr,
            key,
            content,
            original_line,
            line_num,
            usage: vec![],
        }
    }
}

impl DebugData {
    pub fn new(addr: u16, key: String, content: Vec<Vec<u8>>, original_line: String, line_num: usize) -> Self {
        DebugData {
            addr,
            key,
            content,
            original_line,
            line_num,
            usage: vec![],
        }
    }
}

impl DebugLabel {
    pub fn new(byte: u16, name: String, original_line: String, line_num: usize) -> Self {
        DebugLabel {
            byte,
            name,
            original_line,
            line_num,
            usage: vec![],
        }
    }
}

impl DebugUsage {
    pub fn new(op_byte: u16, offset: u8, line: usize) -> Self {
        DebugUsage { op_addr: op_byte, offset, line }
    }
}
