use std::fs;
use std::os::unix::net::{UnixListener, UnixStream};

pub mod redis;

fn handle_client(_stream: UnixStream) {
    println!("User connected.");
    /*let mut parser = redis::Parser::new(BufReader::new(stream));
    loop {
        match parser.parse_value() {
            Ok(value) => {
                result = handle_command(value);
                println!("{?:} -> {?:}", value, result);
            }
            Err(err) => {
                println!("Failed to read result: {}", err);
                return;
            }
        };
    }*/
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
