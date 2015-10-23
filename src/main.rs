extern crate iron;
#[macro_use]
extern crate router;
extern crate url;
extern crate bodyparser;
extern crate rustc_serialize;
extern crate rand;
extern crate rusqlite;
extern crate time;

use iron::prelude::*;
use iron::{ Url, status, Headers };
use iron::request::{ Body };
use iron::middleware::{BeforeMiddleware, AfterMiddleware};
use iron::typemap::Key;
use iron::modifiers::Redirect;
use router::Router;
use url::Host;
use rand::os::OsRng;
use rand::Rng;
use rusqlite::{SqliteConnection, SqliteOpenFlags, SqliteResult};
use time::get_time;
use time::Timespec;

#[derive(Debug, Clone, RustcDecodable)]
struct Register {
    key: String,
    invite: String,
    url: String
}

#[derive(Debug, Clone, RustcDecodable)]
struct Reconfigure {
    key: String,
    signature: String,
    payload: String
}

#[derive(Debug)]
struct Token {
    id: i64,
    time_created: Timespec
}

fn open_db_connection() -> SqliteResult<SqliteConnection> {
    SqliteConnection::open("bastet.db")
}

fn main() {
    // Open new sqlite connection with a flag to allow multi-threading
    let conn = open_db_connection().expect("Failed to open db connection (main)");
    conn.execute("CREATE TABLE IF NOT EXISTS tokens (
        id              INTEGER PRIMARY KEY,
        time_created    INTEGER
    )", &[]).unwrap();

    let router = router!(
        get "/token" => token_handler,
        get "/" => redirect_handler,
        get "*" => redirect_handler,
        put "/" => register_handler,
        patch "/" => reconfigure_handler
    );

    let mut chain = Chain::new(router);
    chain.link_before(SqliteConnector);

    Iron::new(chain).http("0.0.0.0:4000").expect("Failed to start server");

}

struct SqliteConnector;
struct ConnectionKey;
impl Key for ConnectionKey {
    type Value = SqliteConnection;
}

impl BeforeMiddleware for SqliteConnector {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        match (open_db_connection()) {
            Ok(conn) => {
                req.extensions.insert::<ConnectionKey>(conn);
                Ok(())
            },
            Err(err) => Err(IronError::new(err, (status::InternalServerError, "Database Connection Failed")))
        }
    }
}

fn token_handler(req: &mut Request) -> IronResult<Response> {
    match OsRng::new() {
        Ok(mut rng) => {
            let token = rng.next_u64() as i64;
            let conn = req.extensions.get::<ConnectionKey>().expect("No connection found (token_handler)");
            conn.execute("
            INSERT INTO tokens (id, time_created) VALUES ($1, $2)
            ", &[&token, &get_time().sec]).expect("Failed to insert token (token_handler)");
            Ok(Response::with((status::Ok, token.to_string())))
        },
        Err(err) => Ok(Response::with((status::InternalServerError, err.to_string())))
    }
}

fn redirect_handler(req: &mut Request) -> IronResult<Response> {
    // Create a mutable clone of the request url
    let ref incoming_url_ref = req.url;
    let mut incoming_url = incoming_url_ref.clone();
    // Change url host to a looked up address while maintaining the rest of the url
    // TODO: Actually look up the correct address to use
    incoming_url.host = Host::Domain(String::from("github.com"));
    // Setting port ot 80 as default as port is currently unknown
    // TODO: look up correct port to use
    incoming_url.port = 80;
    Ok(Response::with((status::TemporaryRedirect, Redirect(incoming_url))))
}

fn register_handler(req: &mut Request) -> IronResult<Response> {
    let body = req.get::<bodyparser::Struct<Register>>();
    match body {
        Ok(Some(body)) => {
            if !check_invite(&body.invite) {
                return Ok(Response::with((status::Forbidden, "Invalid Invite Code")))
            }

            if !check_key(&body.key) {
                return Ok(Response::with((status::Forbidden, "Invalid Key")))
            }

            if !check_url(&body.url) {
                return Ok(Response::with((status::BadRequest, "Invalid Url")))
            }

            println!("Parsed Body:\n{:?}", body);
            match save(body.key, body.url) {
                Ok(_) => Ok(Response::with((status::Ok, "Ok"))),
                Err(err) => Ok(Response::with((status::InternalServerError  , err)))
            }
        },
        Ok(None) => {
            println!("Empty Body");
            Ok(Response::with((status::NoContent, "No Content Provided")))
        },
        Err(err) => {
            println!("Error: {:?}", err);
            Ok(Response::with((status::BadRequest, "Failed to parse body")))
        }
    }
}

fn reconfigure_handler(req: &mut Request) -> IronResult<Response> {
    let body = req.get::<bodyparser::Struct<Reconfigure>>();
    panic!()
}

fn check_invite(code: &String) -> bool {
    return code == "Invite";
}

fn check_key(key: &String) -> bool {
    return key == "1234567890";
}

fn check_url(url: &String) -> bool {
    Url::parse(&url).is_ok()
}

fn save(key: String, url: String) -> Result<(), &'static str>{
    return Err("Database not implemented")
}
