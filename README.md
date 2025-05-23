# GO slash

**GO Slash** is a shortcut manager.

## Usage

In your browser type as URL `go/yourShortcut` and you will be redirected to the associate URL.

Manage shortcuts at `go/` using the wrench next to the search bar or at `go/yourShortcut?no_redirect`.

## Installation

### Server

#### Config

`var_name=default_value` => required?, description

`DATABASE_URL` => required, postgres connection url `postgres://user:pwd@localhost/db_name`

`PORT=8000` => listening port

`ADDR=127.0.0.1` => listening port

`SALT1` => required, hash salt used for auth put random chars and remember it

`SALT2` => required, hash salt used for auth put random chars, remember it and keep it secret

`DB_MIGRATE=false` => run db migrations at startup if there is some

#### Run

In `web/` run `cargo build -r` then you got in `target\release` you got the app as `go_web` that you can launch with you env vars setup.
(You also can use directyly `cargo run` if you don't need release compile)

eg.

`VARS... go_web` or `DATABASE_URL=postgres://user:pwd@localhost/db_name SALT1=random1 SALT2=random2 cargo run`

#### Manage DB

1. Open a terminal or command prompt.
2. Connect to PostgreSQL using the superuser postgres on the specified port:
   psql -h localhost -p 6543 -U postgres
3. Create a new user 'go_user' with password 'go_pwd':
   CREATE USER go_user WITH PASSWORD 'go_pwd';
4. Create a new database named 'go':
   CREATE DATABASE go;
5. Grant all privileges on the new database to 'go_user':
   GRANT ALL PRIVILEGES ON DATABASE go TO go_user;
6. Verify that 'go_user' can successfully connect and has the correct privileges.

For now only postgres is available.
Use [diesel](https://diesel.rs/) cli to manage db migrations.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

[MIT](https://choosealicense.com/licenses/mit/)



