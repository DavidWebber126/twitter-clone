use postgres::{Client, NoTls, Row};
use std::fs;

pub struct Posts {
    posts: Vec<Row>
}

impl Posts {
    pub fn new(posts: Vec<Row>) -> Posts {
        Posts { posts }
    }

    pub fn html(self) -> &'static str {
        let html_template = fs::read_to_string("twit-home.html").unwrap();
        let post_html = self.append_posts();

        let html = html_template.replace("    <!--REPLACE-->\r\n", &post_html);

        fs::write("twit-home-generated.html", html).unwrap();        
        "twit-home-generated.html"
    }

    fn append_posts(&self) -> String {
        let mut post_html = String::new();

        for post in self.posts.iter() {
            let author: String = post.get(2);
            let content: String = post.get(3);

            let text = format!("    <p>{} - {}</p>\r\n", content, author);
            post_html.push_str(&text);
        }

        post_html
    }

    pub fn query_following_posts(id: i32) -> Vec<Row> {
        let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
        let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");

        let query = 
        "SELECT post_id, author_id, username AS author_name, content
        FROM posts
        LEFT JOIN followers
            ON author_id = followee_id
        LEFT JOIN users
            ON author_id = user_id
        WHERE follower_id = $1";

        let query_results = client.query(
            query,
            &[&id]
        ).unwrap();

        query_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_data() -> [i32; 3] {
        let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
        let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");

        let follower_id = client.query("INSERT INTO users (username) VALUES('test_follower') RETURNING user_id", &[]).unwrap().pop().unwrap().get(0);

        let author_id1 = client.query("INSERT INTO users (username) VALUES('test_author1') RETURNING user_id", &[]).unwrap().pop().unwrap().get(0);

        let author_id2 = client.query("INSERT INTO users (username) VALUES('test_author2') RETURNING user_id", &[]).unwrap().pop().unwrap().get(0);

        
        let follow1 = client.execute("INSERT INTO followers VALUES ($1, $2)", &[&follower_id, &author_id1]).unwrap();
        assert!(follow1 == 1);
        let follow2 = client.execute("INSERT INTO followers VALUES ($1, $2)", &[&follower_id, &author_id2]).unwrap();
        assert!(follow2 == 1);

        let post1 = client.execute("INSERT INTO posts (author_id, content) VALUES ($1, 'This is the first post')", &[&author_id1]).unwrap();
        assert!(post1 == 1);
        let post2 = client.execute("INSERT INTO posts (author_id, content) VALUES ($1, 'This is the second post')", &[&author_id2]).unwrap();
        assert!(post2 == 1);

        [follower_id, author_id1, author_id2]
    }

    fn delete_test_data(ids: [i32; 3]) {
        let connection_string = "host=localhost port=5432 dbname=Twit-Clone-Project user=postgres password=daVidtEen14";
        let mut client = Client::connect(connection_string, NoTls).expect("Could not connect to postgres");

        let follower_id = ids[0];
        let author_id1 = ids[1];
        let author_id2 = ids[2];

        let delete_post1 = client.execute("DELETE FROM posts WHERE author_id = $1 AND content = 'This is the first post'", 
        &[&author_id1]).unwrap();
        assert!(delete_post1 != 0);

        let delete_post2 = client.execute("DELETE FROM posts WHERE author_id = $1 AND content = 'This is the second post'", 
        &[&author_id2]).unwrap();
        assert!(delete_post2 != 0);

        let remove_following1 = client.execute("DELETE FROM followers WHERE follower_id = $1 AND followee_id = $2", &[&follower_id, &author_id1]).unwrap();
        assert!(remove_following1 != 0);

        let remove_following2 = client.execute("DELETE FROM followers WHERE follower_id = $1 AND followee_id = $2", &[&follower_id, &author_id2]).unwrap();
        assert!(remove_following2 != 0);

        let remove_follower = client.execute("DELETE FROM users WHERE user_id = $1", &[&follower_id]).unwrap();
        assert!(remove_follower != 0);
        
        let remove_author1 = client.execute("DELETE FROM users WHERE user_id = $1", &[&author_id1]).unwrap();
        assert!(remove_author1 != 0);

        let remove_author2 = client.execute("DELETE FROM users WHERE user_id = $1", &[&author_id2]).unwrap();
        assert!(remove_author2 != 0);
    }

    #[test]
    fn html_creation() {
        let ids = test_data();
        let posts = Posts::new(Posts::query_following_posts(ids[0]));
        let html_template = 
"<!DOCTYPE html>\r
<html lang=\"en\">\r
  <head>\r
    <meta charset=\"utf-8\">\r
    <title>Twit Home</title>\r
  </head>\r
  <body>\r
    <h1>Home Page</h1>\r
    <p>This will be the twit clone home page, made with Rust!</p>\r
    <p>This is the first post - test_author1</p>\r
    <p>This is the second post - test_author2</p>\r
  </body>\r
</html>";

        let filename = posts.html();
        let html = fs::read_to_string(filename).unwrap();

        delete_test_data(ids);

        assert_eq!(html_template, html);
    }

    #[test]
    fn query_follower_posts() {
        let ids = test_data();
        
        let posts = Posts::query_following_posts(ids[0]);
        for post in posts.iter() {
            let author_id: i32 = post.get(1);
            let content: String = post.get(3);

            assert!((author_id == ids[1]) || (author_id == ids[2]));
            assert!((content == "This is the first post") || (content == "This is the second post"))
        }

        delete_test_data(ids);
    }
}