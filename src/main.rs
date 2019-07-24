use std::fs;
use std::io::{BufReader, Write};
use std::net::TcpStream;
use std::os::unix::net::{UnixListener, UnixStream};

pub mod redis;
use redis::decoder::Decoder;
use redis::encoder::encode_one;

fn handle_client(mut client_stream: UnixStream) {
    println!("User connected.");
    let mut client_decoder = Decoder::new(BufReader::new(
        client_stream.try_clone().expect("Clone failed!"),
    ));

    let mut redis_stream = TcpStream::connect("127.0.0.1:6379").expect("Connection failed!");
    let mut redis_decoder = Decoder::new(BufReader::new(
        redis_stream.try_clone().expect("Clone failed!"),
    ));

    loop {
        match client_decoder.decode_one() {
            Ok(value) => {
                println!("(from client) {:?}", value);
                redis_stream.write(&encode_one(value)).unwrap();
                match redis_decoder.decode_one() {
                    Ok(value) => {
                        println!("(from server) {:?}", value);
                        client_stream
                            .write(&encode_one(value))
                            .expect("Failed to write to client!");
                    }
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
