#![allow(unused_imports)]
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::process::Command;
use std::ptr::null;
use std::sync::{Arc, Mutex};

fn main() { 

    let map: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let map_clone = map.clone();
                std::thread::spawn(move || handle_connection(stream, map_clone));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, map: Arc<Mutex<HashMap<String, String>>>) {
    let mut buffer = [0; 1024];
    loop {        
        let bytes_len = stream.read(&mut buffer).unwrap();
        if bytes_len == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buffer[..bytes_len]);
        let (command, key, value) = parse_command(&request);
    
        match command {
            "PING" => {
                stream.write_all(b"+PONG\r\n".as_ref()).unwrap();
            }
            "ECHO" => {
                let response = format!("${}\r\n{}\r\n", key.len(), key);
                stream.write_all(response.as_bytes()).unwrap();
            }
            "SET" => {
                let mut map = map.lock().unwrap();
                map.insert(key.to_string(), value.to_string());
                stream.write_all(b"+OK\r\n".as_ref()).unwrap();
            }
            "GET" => {
                let map = map.lock().unwrap();
                let value = map.get(key).unwrap_or(&String::new()).to_string();
                let response = format!("${}\r\n{}\r\n", value.len(), value);
                stream.write_all(response.as_bytes()).unwrap();
            }
            _ => {
                stream.write_all(b"-ERR unknown command\r\n".as_ref()).unwrap();
            }
        }
    }
}

fn parse_command(command: &str) -> (&str, &str, &str) {
    let commands = command.split("\r\n").collect::<Vec<&str>>();
    let command = commands.get(2).copied().unwrap_or("");
    let key = commands.get(4).copied().unwrap_or("");
    let value = commands.get(6).copied().unwrap_or("");
    (command, key, value)
}