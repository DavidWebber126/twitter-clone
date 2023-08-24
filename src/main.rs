use std::{
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream}
};
use twitter_clone::{HttpParser, BodyParse};
use postgres::{Client, NoTls, Error};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
}


/// Main function to handle user's connection
/// We want to get the user's id and use this to load their home page
fn handle_connection(stream: TcpStream) {
    let http = HttpParser::new(stream);
    let request_line = http.get_request_line();
    let headers = http.get_headers();
    let body = http.get_body();

    println!("The request line is: {}", request_line);
    println!("The headers are: {}", headers);
    if let Some(body) = body.clone() {
        println!("The body is: {body}");
    }

    let (status_line, filename) = if &request_line[0..3] == "GET" {
        handle_get(request_line)
    } else if &request_line[0..4] == "POST" {
        handle_post(request_line, body.unwrap())
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    let mut stream = http.get_stream();
    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_get(request_line: String) -> (&'static str, &'static str) {
    let (status_line, filename) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "login.html"),
            "GET /signup? HTTP/1.1" => ("HTTP/1.1 200 OK", "signup.html"),
            "GET /homepage HTTP/1.1" => {
                //generate_home();
                ("HTTP/1.1 200 OK", "twit-home.html")
            },
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
        };

    (status_line, filename)
}

fn handle_post(request_line: String, body: String) -> (&'static str, &'static str) {
    let mut values = BodyParse::new(body).body_values();
    let (status_line, filename) = match &request_line[..] {
        "POST /login HTTP/1.1" => {
            // TODO: If username not in database then ask user to retry (add red text to html)
            let username = values.pop().unwrap();
            
            println!("username is: {}", username);
            if username_in_database(username) {
                ("HTTP/1.1 303 See Other\r\nLocation: /homepage", "twit-home.html")
            } else {
                ("HTTP/1.1 200 OK", "login-error.html")
            }
        },
        "POST /signup HTTP/1.1" => {
            match add_user_to_users(values.pop().unwrap()) {
                Ok(_) => ("HTTP/1.1 200 OK", "twit-home.html"),
                Err(_) => ("HTTP/1.1 200 OK", "signup-error.html")
            }
            
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    
    (status_line, filename)
}

fn add_user_to_users(username: String) -> Result<(), Error> {
    let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
    let mut client = Client::connect(connection_string, NoTls)?;

    client.execute(
        "INSERT INTO users (username) VALUES ($1)",
        &[&username]
    )?;

    Ok(())
}

fn username_in_database(username: String) -> bool {
    let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
    let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");
    
    let query_results = client.query(
        "SELECT user_id FROM users WHERE username = $1",
        &[&(username)]
    );

    match query_results {
        Ok(results) => {
            if results.len() >= 1 {
                true
            } else {
                false
            }
        }
        Err(_) => false
    }
}

fn generate_home(username: String) {

}
