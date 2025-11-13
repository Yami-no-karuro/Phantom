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

fn handle_request(
    mut stream: TcpStream, 
    forward_to: Arc<String>, 
    sp_map: Arc<HashMap<String, bool>>
) -> Result<(), io::Error> {
    
    // The request stream has to be read inside a loop because the 
    // TcpStream::[read](https://doc.rust-lang.org/std/net/struct.TcpStream.html#impl-Read-for-%26TcpStream) implementation
    // only pulls **some** bytes in to the buffer, so we should repeat the process until bytes is 0.
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

    let request: String = String::from_utf8_lossy(&request_buffer[..])
        .to_string();
    
    let request_line: &str = line_parser::get_first(&request);
    let request_line_parts: Vec<&str> = line_parser::get_parts(request_line);

    // GET /api/v1/example HTTP/1.1\r\n
    // [0][Method] [1][Path] ...
    // [0][GET]    [1][/api/v1/example] ...
    let request_method: &str = request_line_parts[0];
    let request_path: &str = request_line_parts[1];

    if sp_map.contains_key(request_path) {
        println!("Request ([{}] - {}) has been blocked. Request path scored positive in the sensitive path database.", request_method, request_path);

        let response: &str = "HTTP/1.1 401 Forbidden\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        return Ok(());
    }

    let mut proxy_stream: TcpStream = TcpStream::connect(&*forward_to)?;
    proxy_stream.write_all(&request_buffer)?;
    proxy_stream.flush()?;

    // The response from the proxy stream has to be read inside a loop because the 
    // TcpStream::[read](https://doc.rust-lang.org/std/net/struct.TcpStream.html#impl-Read-for-%26TcpStream) implementation
    // only pulls **some** bytes in to the buffer, so we should repeat the process until bytes is 0.
    let mut response_buffer: Vec<u8> = Vec::new();
    let mut res_chunk: [u8; RESPONSE_BUFF] = [0; RESPONSE_BUFF];
    loop {
        let bytes: usize = proxy_stream.read(&mut res_chunk)?;
        if bytes == 0 {
            break;
        }
        
        response_buffer.extend_from_slice(&res_chunk[..bytes]);
        if bytes < RESPONSE_BUFF {
            break;
        }
    }

    stream.write_all(&response_buffer)?;
    stream.flush()?;
    return Ok(());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Error: \"Invalid arguments!\"");
        eprintln!("Usage: \"{} <port> <forward_to>\"", args[0]);
        process::exit(1);
    }

    let port: &str = &args[1];
    let forward_to: &str = &args[2];
    let address: String = format!("127.0.0.1:{}", port);
    let listener: TcpListener = TcpListener::bind(address)
        .unwrap();

    // We need to use [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html) to share sp_map across threads.
    // This is required, but we don't need [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) and locks because we're in read-only context 
    // and we don't have to worry about race-conditions.
    let sp_map: HashMap<String, bool> = source_loader::load_source("source/sensitive-paths.txt").unwrap();
    let shared_sp_map: Arc<HashMap<String, bool>> = Arc::new(sp_map);
    let forward_to: Arc<String> = Arc::new(forward_to.to_string());

    println!("Phantom proxy will be available soon!");
    println!("127.0.0.1:{} -> {}", port, forward_to);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        let shared_sp_map_clone: Arc<HashMap<String, bool>> = Arc::clone(&shared_sp_map);
        let forward_to_clone: Arc<String> = Arc::clone(&forward_to);
        
        thread::spawn(move || {
            if let Err(e) = handle_request(stream, forward_to_clone, shared_sp_map_clone) {
                eprintln!("Error: \"{}\".", e);
            }
        });
    }
}
