#![allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::process::Command;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    loop {        
        let bytes_len = stream.read(&mut buffer).unwrap();
        if bytes_len == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buffer[..bytes_len]);
        println!("Request: {}", request.to_ascii_uppercase().trim());
    
        let command = request.to_ascii_uppercase().trim().to_string();
        if command.contains("PING") {
            stream.write_all(b"+PONG\r\n".as_ref()).unwrap();
            println!("Response:PONG");
        } else {
            stream.write_all(b"+PONG\r\n".as_ref()).unwrap();
        }
    }
}
