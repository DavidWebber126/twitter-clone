CREATE TABLE users (
	user_id serial PRIMARY KEY,
	username varchar(20) NOT NULL
);

CREATE TABLE posts (
	post_id serial PRIMARY KEY,
	author_id int NOT NULL,
	FOREIGN KEY(author_id)
		REFERENCES users (user_id),
	content varchar(250)
);

CREATE TABLE sessions (
	user_id int PRIMARY KEY,
	session_id int NOT NULL,
	time_created timestamp NOT NULL,
	FOREIGN KEY(user_id)
		REFERENCES users (user_id)
);