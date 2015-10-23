extern crate iron;
#[macro_use]
extern crate router;
extern crate url;
extern crate bodyparser;
extern crate rustc_serialize;

use iron::prelude::*;
use iron::{ Url, status, Headers };
use iron::request::{ Body };
use iron::modifiers::Redirect;
use router::Router;
use url::Host;

#[derive(Debug, Clone, RustcDecodable)]
struct Register {
    key: String,
    invite: String,
    url: String
}

fn main() {
    let router = router!(
        get "/" => redirect_handler,
        get "*" => redirect_handler,
        put "/" => register_handler
    );

    Iron::new(router).http("0.0.0.0:4000").expect("Failed to start server");

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
                Err(err) => Ok(Response::with((status::InternalServerError, err)))
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
