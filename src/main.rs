use std::{
    str,
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
};
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;
use postgres::{Client, NoTls, Error};

pub mod http;
pub mod posts;
use crate::http::{HttpParser, CookieParse, BodyParse};
use crate::posts::Posts;


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

    let (status_line, headers, filename) = if &request_line[0..3] == "GET" {
        handle_get(request_line, headers)
    } else if &request_line[0..4] == "POST" {
        handle_post(request_line, body.unwrap())
    } else {
        ("HTTP/1.1 404 NOT FOUND", "".to_string(), "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = match headers.len() {
        0 => format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}"),
        _ => format!("{status_line}\r\n{headers}\r\nContent-Length: {length}\r\n\r\n{contents}"),
    };

    let mut stream = http.get_stream();
    stream.write_all(response.as_bytes()).unwrap();
}

fn handle_get(request_line: String, headers: String) -> (&'static str, String, &'static str) {
    let mut cookies = CookieParse::new(headers).cookie_values();
    let (status_line, header, filename) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "".to_string(), "login.html"),
            "GET /signup? HTTP/1.1" => ("HTTP/1.1 200 OK", "".to_string(), "signup.html"),
            "GET /homepage HTTP/1.1" => {
                match valid_session(cookies.pop().unwrap()) {
                    Some(id) => {
                        let html = make_homepage(id);
                        println!("YES!");
                        ("HTTP/1.1 200 OK", "".to_string(), html)
                    },
                    None => ("HTTP/1.1 303 See Other", "Location: /".to_string(), "twit-home.html")
                }
                
            },
            _ => ("HTTP/1.1 404 NOT FOUND", "".to_string(), "404.html")
        };

    (status_line, header, filename)
}

fn handle_post(request_line: String, body: String) -> (&'static str, String, &'static str) {
    let mut values = BodyParse::new(body).body_values();
    let (status_line, headers, filename) = match &request_line[..] {
        "POST /login HTTP/1.1" => {
            // TODO: If username not in database then ask user to retry (add red text to html)
            let username = values.pop().unwrap();

            match get_user_id(&username) {
                Some(id) => {
                    let cookie = create_session(id).unwrap();
                    let html = make_homepage(id);
                    ("HTTP/1.1 303 See Other",
                    format!("Location: /homepage\r\nSet-Cookie: id={cookie}; Secure; HttpOnly"),
                    html)
                } 
                None => {
                    ("HTTP/1.1 200 OK", "".to_string(), "login-error.html")   
                }
            }
        },
        "POST /signup HTTP/1.1" => {
            match add_user_to_users(values.pop().unwrap()) {
                Ok(id) => {
                    let cookie = create_session(id).unwrap();
                    let html = make_homepage(id);
                    (
                        "HTTP/1.1 303 See Other",
                        format!("Location: /homepage\r\nSet-Cookie: id={cookie}; Secure; HttpOnly"),
                        html
                    )
                },
                Err(_) => ("HTTP/1.1 200 OK", "".to_string(), "signup-error.html")
            }
            
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "".to_string(), "404.html")
    };
    
    (status_line, headers, filename)
}

fn add_user_to_users(username: String) -> Result<i32, Error> {
    let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
    let mut client = Client::connect(connection_string, NoTls)?;

    let mut id = client.query(
        "INSERT INTO users (username) VALUES ($1) RETURNING user_id",
        &[&username]
    )?;

    Ok(id.pop().unwrap().get(0))
}

fn get_user_id(username: &String) -> Option<i32> {
    let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
    let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");
    
    let mut query_results = client.query(
        "SELECT user_id FROM users WHERE username = $1",
        &[username]
    ).unwrap();

    match query_results.len() {
        0 => None,
        1 => {
            let id: i32 = query_results.pop().unwrap().get(0);
            Some(id)
        },
        _ => panic!("User id query returned more than one result")
    }
}

fn valid_session(session_id: String) -> Option<i32> {
    let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
    let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");

    let mut query_results = client.query(
        "SELECT user_id FROM sessions WHERE session_id = $1",
        &[&session_id]
    ).unwrap();

    
    match query_results.len() {
        0 => None,
        1 => {
            let id: i32 = query_results.pop().unwrap().get(0);
            Some(id)
        }
        _ => panic!("Session id query returned multiple rows")
    }


}

fn create_session(user_id: i32) -> Result<String, Error>{
    let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
    let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");

    let mut rng = StdRng::from_entropy();
    let mut result: Vec<u8> = vec![0; 5];

    rng.fill(&mut result[..]);

    let session_id: String = result.iter().map(|int| int.to_string()).collect();

    client.execute(
        "INSERT INTO sessions (user_id, session_id, time_created)
        VALUES ($1, $2, CURRENT_TIMESTAMP)
        ON CONFLICT (user_id)
        DO UPDATE SET session_id = $2, time_created = CURRENT_TIMESTAMP",
        &[&user_id, &session_id]
    )?;

    Ok(session_id)
}

// Get list of posts made by followers
fn make_homepage(id: i32) -> &'static str {
    let posts = Posts::new(Posts::query_following_posts(id));
    posts.html()
}
