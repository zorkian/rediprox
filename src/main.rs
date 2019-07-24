use std::fs;
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};

pub mod redis;
use redis::decoder::Decoder;
use redis::encoder::encode_one;

fn handle_client(client_stream: UnixStream) {
    println!("User connected.");
    let mut client_decoder = Decoder::new(BufReader::new(client_stream));

    let mut redis_stream = TcpStream::connect("127.0.0.1:6379").unwrap();
    let mut redis_decoder = Decoder::new(BufReader::new(&redis_stream));

    loop {
        match client_decoder.decode_one() {
            Ok(value) => {
                println!("(from client) {:?}", value);
                redis_stream.write(&encode_one(value)).unwrap();
                match redis_decoder.decode_one() {
                    Ok(value) => {}
                    Err(err) => {
                        println!("Error from Redis: {}", err);
                        return;
                    }
                }
            }
            Err(err) => {
                println!("Failed to read result: {}", err);
                return;
            }
        };
    }
}

fn setup_listener() -> Result<UnixListener, std::io::Error> {
    // If file doesn't exist, don't care
    let _ = fs::remove_file("/tmp/rediprox.socket");
    return UnixListener::bind("/tmp/rediprox.socket");
}

fn main() -> Result<(), std::io::Error> {
    let listener = setup_listener()?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(err) => {
                println!("Failed to read from stream: {}", err);
                break;
            }
        }
    }

    Ok(())
}
