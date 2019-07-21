use crate::redis::RedisValue;
use std::fmt;
use std::io::{Bytes, Read};

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
        match self.bytes.next() {
            36 => self.decode_bulkstring(),
            _ => Err(DecodeError),
        }
    }

    // Read up until a \r\n.
    fn read_line(&mut self) -> Result<Vec<u8>, DecodeError> {
        let mut result: Vec<u8> = Vec::new();
        loop {
            match self.bytes.next()? {
                13 => {
                    if self.bytes.next()? == 10 {
                        return Ok(result);
                    } else {
                        return Err(DecodeError);
                    }
                }
            }
        }
    }

    fn decode_bulkstring(&mut self) -> Result<RedisValue, DecodeError> {
        Err(DecodeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::redis::encoder::encode;

    #[test]
    fn test_decoder() {
        let vec = encode(RedisValue::Null);
        let mut decoder = Decoder::new(&vec[..]);
        assert_eq!(decoder.decode_one().unwrap(), RedisValue::Null)
    }
}
