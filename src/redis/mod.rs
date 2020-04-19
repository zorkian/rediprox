use std::str;

pub mod decoder;
pub mod encoder;

// All possible returnable Redis value types.
#[derive(Debug, PartialEq, Clone)]
pub enum RedisValue {
    Integer(i64),
    String(String),
    BulkString(Vec<u8>),
    Error(String),
    Null,
    Array(Vec<RedisValue>),
}

impl RedisValue {
    pub fn abuse(&self) -> String {
        match self {
            RedisValue::Integer(val) => format!("{}", val),
            RedisValue::String(val) => format!("{}", val),
            RedisValue::Null => format!("null"),
            RedisValue::Error(val) => format!("Error({})", val),
            RedisValue::BulkString(val) => {
                format!("{}", str::from_utf8(val).unwrap_or(&*format!("{:?}", val)))
            }
            RedisValue::Array(vals) => {
                let mut strs: Vec<String> = Vec::new();
                for val in vals {
                    strs.push(val.abuse());
                }
                return strs.join(" ");
            }
        }
    }
}
