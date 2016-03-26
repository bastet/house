extern crate iron;
extern crate url;
extern crate bodyparser;
extern crate rustc_serialize;
extern crate rand;
extern crate rusqlite;
extern crate time;

use iron::prelude::*;
use iron::{ Url, status };
use iron::modifiers::Redirect;
use url::Host;
use rand::os::OsRng;
use rand::Rng;
use time::get_time;
use std::io::{ Error, ErrorKind };
use std::net::SocketAddr::{ V4, V6 };
use std::error;
use models::{ Register, Reconfigure, ConnectionKey, IpMapping };
use rusqlite::{ SqliteConnection };

fn iron_err_from_string(err: &str) -> IronResult<Response> {
    Err(IronError::new(Error::new(ErrorKind::Other, err), status::InternalServerError))
}

fn iron_err_from_error<T>(err: T) -> IronResult<Response>
    where T : Into<Box<error::Error + Send + Sync>>
{
    Err(IronError::new(Error::new(ErrorKind::Other, err), status::InternalServerError))
}

/// # Generate new request tokens
/// other API endpoints require a valid token stored in the database, hitting this endpoint generates and returns a new token (as well as stores it in the database for later validation)
pub fn token_handler(req: &mut Request) -> IronResult<Response> {
    //Create a RNG, early exit if fail
    let mut rng = match OsRng::new() {
        Ok(rng) => rng,
        Err(err) => return Err(IronError::new(err, status::InternalServerError))
    };

    //Get the SQL connection, early exit if fail
    let conn = match req.extensions.get::<ConnectionKey>() {
        Some(conn) => conn,
        None => return iron_err_from_string("No SQL Connection")
    };

    //Generate a completely random token
    //We sacrifice 1 bit of randomness to ensure tokens are always positive
    let token = rng.gen::<i64>().abs();

    //Insert token into DB
    match conn.execute("INSERT INTO tokens (id, time_created) VALUES ($1, $2)", &[&token, &get_time().sec]) {

        //If we inserted zero values something is wrong!
        Ok(0) => iron_err_from_string("Failed to insert token into DB"),

        //Inserted some number of values, cool
        Ok(_) => Ok(Response::with((status::Ok, token.to_string()))),

        //Error!
        Err(err) => Err(IronError::new(err, status::InternalServerError))
    }
}

// # Redirect all requests to the appropriate hub
// When a request comes in which is not to a known local endpoint we lookup the correct hub to redirect to (by IP address) and redirect to there
pub fn redirect_handler(req: &mut Request) -> IronResult<Response> {

    //Get the SQL connection, early exit if fail
    let conn = match req.extensions.get::<ConnectionKey>() {
        Some(conn) => conn,
        None => return iron_err_from_string("No SQL Connection")
    };

    //Prepare a query to lookup IP
    let mut stmt = match conn.prepare("SELECT internal_ip, internal_port FROM redirects WHERE public_ip = ?") {
        Err(err) => return iron_err_from_error(err),
        Ok(stmt) => stmt
    };

    //Convert IP address to a string
    let ip = match req.remote_addr {
        V4(addr) => addr.ip().to_string(),
        V6(addr) => addr.ip().to_string()
    };

    //Lookup the correct URL to redirect to based on the origin IP
    let optional_addr = match stmt.query_map(&[&ip], |row| { IpMapping { ip: row.get::<String>(0), port: row.get::<i32>(1) } } ) {
        Err(err) => return iron_err_from_error(err),
        Ok(mut result) => result.nth(0)
    };

    //The nth item may not exist, so we need to unwrap that
    let addr_result = match optional_addr {
        None => return Err(IronError::new(Error::new(ErrorKind::Other, "No mapping found for this address"), status::NotFound)),
        Some(addr) => addr
    };

    //The nth item may have been an error, so we need to unwrap that
    let addr = match addr_result {
        Err(err) => return iron_err_from_error(err),
        Ok(result) => result
    };

    // Create a mutable clone of the request url
    // Change url host to a looked up address while maintaining the rest of the request
    let mut incoming_url = req.url.clone();
    incoming_url.host = Host::Domain(addr.ip);
    incoming_url.port = addr.port as u16;

    Ok(Response::with((status::TemporaryRedirect, Redirect(incoming_url))))
}

// # Register a new redirect
// Add a new redirect to the database, requires a valid invite code to already exist in the database
pub fn register_handler(req: &mut Request) -> IronResult<Response> {

    //Parse the body
    let body = match req.get::<bodyparser::Struct<Register>>() {
        Ok(Some(body)) => body,
        Ok(None) => return Ok(Response::with((status::NoContent, "No Content Provided"))),
        Err(err) => return iron_err_from_error(err),
    };

    //Get the SQL connection, early exit if fail
    let conn = match req.extensions.get::<ConnectionKey>() {
        Some(conn) => conn,
        None => return iron_err_from_string("No SQL Connection")
    };

    if !check_invite(&conn, &body.invite) {
        return Ok(Response::with((status::Forbidden, "Invalid Invite Code")))
    }

    if !check_key(&body.key) {
        return Ok(Response::with((status::Forbidden, "Invalid Key")))
    }

    if !check_url(&body.url) {
        return Ok(Response::with((status::BadRequest, "Invalid Url")))
    }

    //Insert a new user into the database
    match conn.execute("INSERT INTO users (public_key) VALUES ($1)", &[&body.key]) {
        Ok(1) => Ok(Response::with((status::Ok, "Created user"))),
        Ok(_) => iron_err_from_string("Failed to create new user"),
        Err(err) => { println!("{:?}", err); iron_err_from_error(err) }
    }
}

fn check_invite(conn: &SqliteConnection, code: &String) -> bool {
    //todo: Implement invites (here we need to check if the given invite code exists in the database)
    return code == "Invite";
}

fn check_key(key: &String) -> bool {
    //todo: Implement keys (here, we just need to check that the key is a valid key)
    true
}

fn check_url(url: &String) -> bool {
    Url::parse(&url).is_ok()
}

pub fn reconfigure_handler(req: &mut Request) -> IronResult<Response> {
    let body = req.get::<bodyparser::Struct<Reconfigure>>();
    panic!()
}
