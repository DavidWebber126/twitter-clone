use std::io::prelude::*;
use std::net::TcpStream;
use std::str;

pub struct HttpParser {
    stream: TcpStream,
    request_line: String,
    headers: String,
    body: Option<String>
}

impl HttpParser {
    pub fn new(mut stream: TcpStream) -> HttpParser {
        let (request_line, headers, body) = HttpParser::parse(&mut stream);

        HttpParser{ stream, request_line, headers, body }
    }

    fn parse(stream: &mut TcpStream) -> (String, String, Option<String>) {
        let mut buffer: Vec<u8> = Vec::new();
        let (mut request_line, mut headers) = ("".to_string(), "".to_string());
        let mut crlf_checker: Vec<u8> = Vec::with_capacity(4);
        let mut body_start_index = 0;
        let mut body_length: usize = 0;
        let mut counter = 0;

        for result in stream.bytes() {
            match result {
                Ok(byte) => {
                    buffer.push(byte);
                    if (byte == b'\r') | (byte == b'\n') {
                        crlf_checker.push(byte);
                    } else {
                        crlf_checker.clear();
                    }
                },
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
            }

            if body_start_index != 0 {
                counter += 1;
            }

            if (counter >= body_length) & (body_start_index != 0) {
                break
            }

            if crlf_checker == vec![b'\r', b'\n', b'\r', b'\n'] {
                body_start_index = buffer.len();
                (request_line, headers) = HttpParser::parse_head(&buffer);
                let content_length: Option<usize> = HttpParser::parse_content_length(&headers);
                println!("The content length is: {:?}", content_length);
                buffer.clear();

                match content_length {
                    Some(length) => body_length = length,
                    None => break
                }
            }
        }

        let mut body = None;
        if body_length != 0 {
            body = Some(str::from_utf8(&buffer).unwrap().to_string());
        }

        (request_line, headers, body)
        
    }

    fn parse_head(head_buffer: &Vec<u8>) -> (String, String) {
        let head = str::from_utf8(&head_buffer).unwrap();
        let mut request_line = "".to_string();
        let mut headers = "".to_string();

        for (index, line) in head.lines().enumerate() {
            if index == 0 {
                request_line.push_str(line);
            } else if (line != "\n") | (line != "\r") {
                headers.push_str(line);
                headers.push_str("\n");
            }
        };

        (request_line, headers)
    }

    fn parse_content_length(headers: &String) -> Option<usize> {
        let content_length_line: String = headers.lines()
            .filter(|line| line.contains("Content-Length"))
            .collect();

        println!("content line is: {}", content_length_line);

        let content_length = content_length_line.split(": ").last().unwrap();
        let content_length: Result<usize, _> = content_length.parse();

        match content_length {
            Ok(length) => Some(length),
            Err(_) => None
        }
    }

    pub fn get_stream(self) -> TcpStream {
        self.stream
    }

    pub fn get_request_line(&self) -> String {
        self.request_line.clone()
    }

    pub fn get_headers(&self) -> String {
        self.headers.clone()
    }

    pub fn get_body(&self) -> Option<String> {
        self.body.clone()
    }
}

pub struct BodyParse{
    body: String
}

impl BodyParse {
    pub fn new(body: String) -> BodyParse {
        BodyParse { body }
    }

    pub fn body_values(&self) -> Vec<String> {
        let values: Vec<String> = self.get_body()
            .split("&")
            .map(|form| form.split("=").last().expect("Form didn't have a value").to_string())
            .collect();

        println!("{:?}", values);
        values     
    }

    fn get_body(&self) -> &str {
        self.body.as_str()
    }
}