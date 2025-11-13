use std::env;
use std::thread;
use std::process;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::collections::HashMap;

mod line_parser;
mod source_loader;

const REQUEST_BUFF: usize = 2048;
const RESPONSE_BUFF: usize = 2048;

fn handle_request(mut stream: TcpStream, sp_map: Arc<HashMap<String, bool>>) -> Result<(), io::Error> {
    let mut request_buffer: Vec<u8> = Vec::new();
    let mut chunk: [u8; REQUEST_BUFF] = [0; REQUEST_BUFF];

    loop {
        let bytes_read: usize = stream.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }
        
        request_buffer.extend_from_slice(&chunk[..bytes_read]);
        if bytes_read < REQUEST_BUFF {
            break;
        }
    }

    let request: String = String::from_utf8_lossy(&request_buffer[..]).to_string();
    let request_lines: Vec<&str> = line_parser::get_all(&request);
    let request_line_parts: Vec<&str> = line_parser::get_parts(&request_lines[0]);

    // GET /api/v1/example HTTP/1.1\r\n
    // [0][Method] [1][Path] ...
    // [0][GET]    [1][/api/v1/example] ...
    let request_method: &str = request_line_parts[0];
    let request_path: &str = request_line_parts[1];

    let mut request_body: String = String::new();
    let mut request_headers: Vec<&str> = Vec::new();
    let mut is_body: bool = false;

    for line in &request_lines[1..] {
        if line.is_empty() {
            is_body = true;
            continue;
        }

        if is_body {
            request_body.push_str(line);
            request_body.push('\n');
        } else {
            request_headers.push(line);
        }
    }

    if sp_map.contains_key(request_path) {
        println!("Request ([{}] - {}) was blocked!", request_method, request_path);

        let response: &str = "HTTP/1.1 401 Forbidden\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        return Ok(());
    }

    let mut proxy_stream: TcpStream = TcpStream::connect("127.0.0.1:8080")?;
    proxy_stream.write_all(&request_buffer)?;
    proxy_stream.flush()?;

    let mut response_buffer: Vec<u8> = Vec::new();
    let mut res_chunk: [u8; RESPONSE_BUFF] = [0; RESPONSE_BUFF];

    loop {
        let bytes_read: usize = proxy_stream.read(&mut res_chunk)?;
        if bytes_read == 0 {
            break;
        }
        
        response_buffer.extend_from_slice(&res_chunk[..bytes_read]);
        if bytes_read < RESPONSE_BUFF {
            break;
        }
    }

    stream.write_all(&response_buffer)?;
    stream.flush()?;
    return Ok(());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: \"Invalid arguments!\"");
        eprintln!("Usage: \"{} <port>\"", args[0]);
        process::exit(1);
    }

    let port: &str = &args[1];
    let address: String = format!("127.0.0.1:{}", port);
    let listener: TcpListener = TcpListener::bind(address)
        .unwrap();

    // We need to use [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) to share sp_map across threads.
    // This is required, but we don't need [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) and locks because we're in read-only context 
    // and we don't have to protect ourselves from race-conditions.
    let sp_map: HashMap<String, bool> = source_loader::load_source("source/sensitive-paths.txt").unwrap();
    let shared_sp_map = Arc::new(sp_map);

    println!("Phantom listening on port: {}.", port);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let shared_sp_map_clone = Arc::clone(&shared_sp_map);
        thread::spawn(move || {
            if let Err(e) = handle_request(stream, shared_sp_map_clone) {
                eprintln!("Error: \"{}\".", e);
            }
        });
    }
}
