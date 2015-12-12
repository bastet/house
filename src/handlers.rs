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
use std::io::{Error, ErrorKind};

use models::{ Register, Reconfigure, ConnectionKey, IpMapping };

fn iron_err_from_string(err: &str) -> IronResult<Response> {
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

pub fn redirect_handler(req: &mut Request) -> IronResult<Response> {

    //Get the SQL connection, early exit if fail
    let conn = match req.extensions.get::<ConnectionKey>() {
        Some(conn) => conn,
        None => return iron_err_from_string("No SQL Connection")
    };

    //Prepare a query to lookup IP
    let mut stmt = match conn.prepare("SELECT internal_ip, internal_port FROM redirects WHERE public_ip = ?") {
        Err(err) => return Err(IronError::new(err, status::InternalServerError)),
        Ok(stmt) => stmt
    };

    //Convert IP address to a integer
    let ip = req.remote_addr.to_string();

    //Lookup the correct URL to redirect to based on the origin IP
    let optionalAddr = match stmt.query_map(&[&ip], |row| { IpMapping { ip: row.get::<String>(0), port: row.get::<i32>(1) } } ) {
        Err(err) => return Err(IronError::new(err, status::InternalServerError)),
        Ok(mut result) => result.nth(0)
    };

    //The nth item may not exist, so we need to unwrap that
    let addrResult = match optionalAddr {
        None => return Err(IronError::new(Error::new(ErrorKind::Other, "No mapping found for this address"), status::NotFound)),
        Some(addr) => addr
    };

    //The nth item may have been an error, so we need to unwrap that
    let addr = match addrResult {
        Err(err) => return Err(IronError::new(err, status::InternalServerError)),
        Ok(result) => result
    };

    // Create a mutable clone of the request url
    // Change url host to a looked up address while maintaining the rest of the request
    let mut incoming_url = req.url.clone();
    incoming_url.host = Host::Domain(addr.ip);
    incoming_url.port = addr.port as u16;

    Ok(Response::with((status::TemporaryRedirect, Redirect(incoming_url))))
}

pub fn register_handler(req: &mut Request) -> IronResult<Response> {
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

pub fn reconfigure_handler(req: &mut Request) -> IronResult<Response> {
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
