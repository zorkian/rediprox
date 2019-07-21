use crate::redis::RedisValue;
use std::convert::TryInto;
use std::fmt;
use std::io::{Bytes, ErrorKind, Read};
use std::str;

struct DecodeError;

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decode error, decoder terminated.")
    }
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decode error at {} line {}", file!(), line!())
    }
}

struct Decoder<T> {
    bytes: Bytes<T>,
}

impl<T: Read> Decoder<T> {
    fn new(reader: T) -> Decoder<T> {
        Decoder {
            bytes: reader.bytes(),
        }
    }

    // Decode a single value off of the front of the stream, this will
    // block until a full value is available. If the stream errors or
    // we receive invalid data, DecodeError is returned.
    fn decode_one(&mut self) -> Result<RedisValue, DecodeError> {
        match self.read_byte()? {
            36 => self.decode_bulkstring(),
            42 => self.decode_array(),
            43 => self.decode_string(),
            45 => self.decode_error(),
            58 => self.decode_integer(),
            _ => Err(DecodeError),
        }
    }

    fn decode_integer(&mut self) -> Result<RedisValue, DecodeError> {
        match str::from_utf8(&self.read_line()?) {
            Ok(val) => match str::parse::<i64>(val) {
                Ok(integer) => Ok(RedisValue::Integer(integer)),
                Err(_) => Err(DecodeError),
            },
            Err(_) => Err(DecodeError),
        }
    }

    fn decode_string(&mut self) -> Result<RedisValue, DecodeError> {
        match str::from_utf8(&self.read_line()?) {
            Ok(val) => Ok(RedisValue::String(val.to_string())),
            Err(_) => Err(DecodeError),
        }
    }

    fn decode_error(&mut self) -> Result<RedisValue, DecodeError> {
        // TODO: Support the 'first word' error kinds which Redis uses?
        match str::from_utf8(&self.read_line()?) {
            Ok(val) => Ok(RedisValue::Error(val.to_string())),
            Err(_) => Err(DecodeError),
        }
    }

    fn decode_bulkstring(&mut self) -> Result<RedisValue, DecodeError> {
        if let RedisValue::Integer(length) = self.decode_integer()? {
            if length == -1 {
                Ok(RedisValue::Null)
            } else if length < 0 {
                // Negative lengths are otherwise not allowed and would be a protocol
                // error.
                Err(DecodeError)
            } else {
                let mut result: Vec<u8> = Vec::with_capacity(length.try_into().unwrap());
                for _ in 0..length {
                    result.push(self.read_byte()?)
                }
                // Consume trailing \r\n, this protocol really feels unnecessary.
                if self.read_byte()? != 13 || self.read_byte()? != 10 {
                    Err(DecodeError)
                } else {
                    Ok(RedisValue::BulkString(result))
                }
            }
        } else {
            Err(DecodeError)
        }
    }

    fn decode_array(&mut self) -> Result<RedisValue, DecodeError> {
        if let RedisValue::Integer(length) = self.decode_integer()? {
            let mut result: Vec<RedisValue> = Vec::with_capacity(length.try_into().unwrap());
            for _ in 0..length {
                result.push(self.decode_one()?)
            }
            Ok(RedisValue::Array(result))
        } else {
            Err(DecodeError)
        }
    }

    // Read up until a \r\n. This function will block until we are able to
    // read a full line.
    fn read_line(&mut self) -> Result<Vec<u8>, DecodeError> {
        let mut result: Vec<u8> = Vec::new();
        loop {
            match self.read_byte()? {
                13 => {
                    if self.read_byte()? == 10 {
                        return Ok(result);
                    } else {
                        // I suppose RESP doesn't disallow an arbitrary \r in the
                        // middle of a string, although it does feel odd.
                        result.push(13);
                    }
                }
                other => result.push(other),
            }
        }
    }

    // Read a single byte and handle retrying if we received an interrupted
    // IO error or the like.
    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        // TODO: This feels like a really verbose way of doing this, I imagine
        // there's a more idiomatic way...?
        loop {
            match self.bytes.next() {
                Some(val) => match val {
                    Ok(val) => return Ok(val),
                    Err(error) => {
                        // Interrupted is specified as being retriable, but other
                        // errors we consider fatal. TODO: is this legit?
                        if error.kind() != ErrorKind::Interrupted {
                            return Err(DecodeError);
                        }
                    }
                },
                None => return Err(DecodeError), // EOF
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::encoder::encode;

    #[test]
    fn test_decoder() {
        let decode_tests = vec![
            RedisValue::Integer(20190717),
            RedisValue::String("I'm a string!".to_string()),
            RedisValue::Error("Oh fudge.".to_string()),
            RedisValue::Null,
            RedisValue::BulkString(vec![0, 13, 10, 4, 8, 15, 16, 23, 42, 0]),
            RedisValue::Array(vec![]),
            RedisValue::Array(vec![
                RedisValue::Integer(20190717),
                RedisValue::String("I'm a string!".to_string()),
                RedisValue::Error("Oh fudge.".to_string()),
                RedisValue::Null,
                RedisValue::BulkString(vec![0, 13, 10, 4, 8, 15, 16, 23, 42, 0]),
                RedisValue::Array(vec![
                    RedisValue::Error("Oh fudge.".to_string()),
                    RedisValue::Error("Oh fudge.".to_string()),
                ]),
            ]),
        ];

        for test in decode_tests {
            let vec = encode(test.clone());
            let mut decoder = Decoder::new(&vec[..]);
            assert_eq!(decoder.decode_one().unwrap(), test)
        }
    }
}
