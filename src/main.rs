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

const BUFF_SIZE: usize = 2048;
const FUZZLIST_PATH: &str = "source/fuzzlist.txt";

fn read_to_buff(
    mut stream: &TcpStream, 
    buff: &mut Vec<u8>
) -> Result<(), io::Error> 
{
    let mut chunk: [u8; BUFF_SIZE] = [0; BUFF_SIZE];
    loop {
        let bytes: usize = stream.read(&mut chunk)?;
        if bytes == 0 { 
            break; 
        }
        
        buff.extend_from_slice(&chunk[..bytes]);
        if bytes < BUFF_SIZE { 
            break; 
        }
    }
    
    return Ok(());
}

fn handle_request(
    mut stream: TcpStream, 
    forward_to: Arc<String>, 
    sp_map: Arc<HashMap<String, bool>>
) -> Result<(), io::Error> 
{
    let mut request_buffer: Vec<u8> = Vec::new();
    read_to_buff(&stream, &mut request_buffer)?;
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
        println!("Request ([{}] - {}) has been blocked.", request_method, request_path);

        let response: &str = "HTTP/1.1 401 Forbidden\r\n\r\n";
        stream.write_all(response.as_bytes())?;
        stream.flush()?;
        return Ok(());
    }

    let proxy: String = format!("127.0.0.1:{}", &*forward_to);
    let mut proxy_stream: TcpStream = TcpStream::connect(proxy)?;
    proxy_stream.write_all(&request_buffer)?;
    proxy_stream.flush()?;

    let mut response_buffer: Vec<u8> = Vec::new();
    read_to_buff(&proxy_stream, &mut response_buffer)?;

    stream.write_all(&response_buffer)?;
    stream.flush()?;
    return Ok(());
}

fn main() 
{
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
    
    let sp_map: HashMap<String, bool> = source_loader::load_source(FUZZLIST_PATH)
        .unwrap();

    let shared_sp_map: Arc<HashMap<String, bool>> = Arc::new(sp_map);
    let forward_to: Arc<String> = Arc::new(forward_to.to_string());

    println!("Phantom Proxy will be available soon!");
    println!("[127.0.0.1:{} -> 127.0.0.1:{}]", port, forward_to);

    for stream in listener.incoming() {
        let stream: TcpStream = stream.unwrap();
        let shared_sp_map_clone: Arc<HashMap<String, bool>> = Arc::clone(&shared_sp_map);
        let forward_to_clone: Arc<String> = Arc::clone(&forward_to);
        
        thread::spawn(move || {
            if let Err(e) = handle_request(stream, forward_to_clone, shared_sp_map_clone) {
                eprintln!("Error: \"{}\".", e);
            }
        });
    }
}
