use std::env;
use std::thread;
use std::process;

use std::io;
use std::io::Read;
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;

use std::collections::HashMap;
use std::fs::File;

use std::net::TcpListener;
use std::net::TcpStream;

mod line_parser;

const REQUEST_BUFF: usize = 4096;

fn load_source(path: &str) -> Result<HashMap<String, bool>, io::Error> {
    let file: File = File::open(&path)?;
    let reader = BufReader::new(file);

    // The source file must contain a single value per row.
    // The lookup table maps the smallest possible value on the left, only for low-complexity search purposes.
    let mut hashmap: HashMap<String, bool> = HashMap::new();
    for line in reader.lines() {
        hashmap.insert(line?, true);
    }

    return Ok(hashmap);
}

fn handle_request(mut stream: TcpStream) -> Result<(), io::Error> {
    let mut request_buffer: [u8; 4096] = [0; REQUEST_BUFF];
    stream.read(&mut request_buffer)?;

    let request: String = String::from_utf8_lossy(&request_buffer[..])
        .to_string();

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

        // Host: foo.bar\r\nContent-Type: application/json...\r\n\r\n
        // An empty line separates the headers from the actual request body.
        // When encountered, content starts to be pushed to the request_body variable.
        if is_body {
            request_body.push_str(line);
            request_body.push('\n');
        } else {
            request_headers.push(line);
        }
    }

    println!("[{}] - {}", request_method, request_path);
    let mut response: &str = "HTTP/1.1 200 OK\r\n\r\n";

    let sensitive_paths_map: HashMap<String, bool> = load_source("source/sensitive-paths.txt")?;
    if sensitive_paths_map.contains_key(request_path) {
        response = "HTTP/1.1 401 Forbidden\r\n\r\n";
    }

    stream.write(response.as_bytes())?;
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

    println!("Phantom listening on port: {}.", port);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            if let Err(e) = handle_request(stream) {
                eprintln!("Error: \"{}\".", e);
            }
        });
    }
}
