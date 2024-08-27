use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use rust_server_book::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    println!("➡️ request: {}", request_line);

    // Determine the file to serve based on the request
    let (status_line, filename, content_type) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html", "text/html"),
        "GET /style.css HTTP/1.1" => ("HTTP/1.1 200 OK", "style.css", "text/css"),
        "GET /sleep HTTP/1.1" => {
            std::thread::sleep(std::time::Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html", "text/html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html", "text/html"),
    };

    let contents = fs::read_to_string(filename).unwrap();

    let content_length = format!("Content-Length: {}", contents.len());

    let response = format!(
        "{status_line}\r\nContent-Type: {content_type}\r\n{content_length}\r\n\r\n{contents}",
    );

    stream.write_all(response.as_bytes()).unwrap();
}
