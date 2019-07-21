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
