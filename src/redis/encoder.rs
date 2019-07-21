use crate::redis::RedisValue;

fn _encode(prefix: u8, body: String) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(body.len() + 3);
    result.push(prefix);
    for byte in body.bytes() {
        result.push(byte);
    }
    result.push(13);
    result.push(10);
    return result;
}

pub fn encode(value: RedisValue) -> Vec<u8> {
    // TODO: Replace the format! calls for itoa purposes with the crate that
    // does it faster.
    match value {
        RedisValue::Null => _encode(36, "-1".to_string()),
        RedisValue::Integer(val) => _encode(58, format!("{}", val)),
        RedisValue::String(val) => _encode(43, val),
        RedisValue::BulkString(val) => {
            let mut result = _encode(36, format!("{}", val.len()));
            result.extend_from_slice(&val);
            result.extend_from_slice(&[13, 10]);
            return result;
        }
        RedisValue::Error(val) => _encode(45, val),
        RedisValue::Array(val) => {
            let mut result = _encode(42, format!("{}", val.len()));
            for subval in val {
                result.extend_from_slice(&encode(subval));
            }
            return result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_encoder() {
        assert_eq!(_encode(1, "abc".to_string()), vec![1, 97, 98, 99, 13, 10]);
        assert_eq!(_encode(2, "".to_string()), vec![2, 13, 10]);
    }

    #[test]
    fn test_string_encode() {
        assert_eq!(
            encode(RedisValue::String("abc".to_string())),
            vec![43, 97, 98, 99, 13, 10]
        );
    }

    #[test]
    fn test_integer_encode() {
        assert_eq!(
            encode(RedisValue::Integer(100)),
            vec![58, 49, 48, 48, 13, 10]
        );
    }

    #[test]
    fn test_bulkstring_encode() {
        assert_eq!(
            encode(RedisValue::BulkString(vec![1, 0, 100, 10])),
            vec![36, 52, 13, 10, 1, 0, 100, 10, 13, 10]
        );
    }

    #[test]
    fn test_error_encode() {
        assert_eq!(
            encode(RedisValue::Error("abc".to_string())),
            vec![45, 97, 98, 99, 13, 10]
        );
    }

    #[test]
    fn test_encode_null() {
        assert_eq!(encode(RedisValue::Null), vec![36, 45, 49, 13, 10])
    }

    #[test]
    fn test_encode_array() {
        assert_eq!(
            encode(RedisValue::Array(vec![
                RedisValue::Null,
                RedisValue::BulkString(vec![0, 1, 2])
            ])),
            vec![42, 50, 13, 10, 36, 45, 49, 13, 10, 36, 51, 13, 10, 0, 1, 2, 13, 10]
        )
    }

    #[test]
    fn test_encode_nested_array() {
        assert_eq!(
            encode(RedisValue::Array(vec![
                RedisValue::BulkString(vec![0, 1, 2]),
                RedisValue::Array(vec![
                    RedisValue::Null,
                    RedisValue::String("abc".to_string()),
                    RedisValue::Array(vec![]),
                ]),
                RedisValue::Integer(1000),
            ])),
            vec![
                42, 51, 13, 10, 36, 51, 13, 10, 0, 1, 2, 13, 10, 42, 51, 13, 10, 36, 45, 49, 13,
                10, 43, 97, 98, 99, 13, 10, 42, 48, 13, 10, 58, 49, 48, 48, 48, 13, 10
            ]
        )
    }
}
