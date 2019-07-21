use crate::redis::RedisValue;
use std::fmt;
use std::io::{Bytes, Read};

struct DecodeError {
    permanent: bool,
}

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
            Some(val) => match val {
                36 => self.decode_bulkstring(),
            },
            _ => Err(DecodeError),
        }
    }

    // Read up until a \r\n.
    fn read_line(&mut self) -> Result<Vec<u8>, DecodeError> {
        let mut result: Vec<u8> = Vec::new();
        loop {
            match self.bytes.next() {
                -1 => Err(DecodeError { permanent: false }),
                13 => {
                    if self.bytes.next().unwrap().unwrap() == 10 {
                        return Ok(result);
                    } else {
                        return Err(DecodeError { permanent: false });
                    }
                }
            }
        }
    }

    // Read a single byte and handle the different states of IO
    // the iterator can be in.
    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        match self.bytes.next() {
            Some(val) => match val {
                Ok(val) => val,
                error => Err(DecodeError { permanent: false }),
            },
            None => Err(DecodeError { permanent: true }), // EOF
        }
    }

    fn decode_bulkstring(&mut self) -> Result<RedisValue, DecodeError> {
        Err(DecodeError { permanent: false })
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
