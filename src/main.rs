#![allow(unused_imports)]
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::process::Command;
use std::ptr::null;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() { 

    let map: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>> = Arc::new(Mutex::new(HashMap::new()));

    let map_clone = map.clone();
    std::thread::spawn(move || {cleanup_expired_keys(map_clone)});
    
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

fn handle_connection(mut stream: TcpStream, map: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>) {
    let mut buffer = [0; 1024];
    loop {        
        let bytes_len = stream.read(&mut buffer).unwrap();
        if bytes_len == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buffer[..bytes_len]);
        let (command, key, value, expiry_type, expiry_value) = parse_command(&request);
    
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
                if expiry_type == "PX" {
                    let expiry_value = expiry_value.parse::<u64>().unwrap();
                    let expiry_time = std::time::Instant::now() + std::time::Duration::from_millis(expiry_value);
                    map.insert(key.to_string(), (value.to_string(), Some(expiry_time)));
                } else if expiry_type == "EX" {
                    let expiry_value = expiry_value.parse::<u64>().unwrap();
                    let expiry_time = std::time::Instant::now() + std::time::Duration::from_secs(expiry_value);
                    map.insert(key.to_string(), (value.to_string(), Some(expiry_time)));
                } else {
                    map.insert(key.to_string(), (value.to_string(), None));
                }
                let response = format!("+OK\r\n");
                stream.write_all(response.as_bytes()).unwrap();
            }
            "GET" => {
                let mut map = map.lock().unwrap();

                let value = match map.get(key) {
                    Some((value, expiry_time)) => (value.to_string(), expiry_time),
                    None => (String::new(), &None),
                };
                if let Some(expiry_time) = value.1 {
                    if *expiry_time < std::time::Instant::now() {
                        map.remove(key);
                        stream.write_all(b"$-1\r\n".as_ref()).unwrap();
                        return;
                    }
                }
                let response = format!("${}\r\n{}\r\n", value.0.len(), value.0);
                stream.write_all(response.as_bytes()).unwrap();
            }
            _ => {
                stream.write_all(b"-ERR unknown command\r\n".as_ref()).unwrap();
            }
        }
    }
}

fn parse_command(command: &str) -> (&str, &str, &str, &str, &str) {
    let commands = command.split("\r\n").collect::<Vec<&str>>();
    let command = commands.get(2).copied().unwrap_or("");
    let key = commands.get(4).copied().unwrap_or("");
    let value = commands.get(6).copied().unwrap_or("");
    let expiry_type = commands.get(8).copied().unwrap_or("");
    let expiry_value = commands.get(10).copied().unwrap_or("");
    (command, key, value, expiry_type, expiry_value)
}

fn cleanup_expired_keys(map: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>) {
    loop {
        {
            let mut map = map.lock().unwrap();
            let now = std::time::Instant::now();

            map.retain(|_, (_, expiry_time)| {
                if let Some(expiry_time) = expiry_time {
                    if *expiry_time < now {
                        return false;
                    }
                }
                true
            });
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
    }   
}