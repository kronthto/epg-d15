use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("1 arg expected")
    }

    let listener = TcpListener::bind(format!("{}:{}", args[1], "7815")).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();

    let input_string = String::from_utf8(buffer.to_vec()).unwrap();
    let input_str = input_string.as_str().trim_matches(char::from(0)).trim();

    let output = Command::new("/root/ed15r")
        .arg(input_str)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        let response;
        println!("error! status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        response = "ERROR".to_string();
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    stream.write(output.stdout.as_slice()).unwrap();
    stream.flush().unwrap();
}
