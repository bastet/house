extern crate iron;
#[macro_use]
extern crate router;
extern crate url;

use iron::prelude::*;
use iron::{ Url, status, Headers };
use iron::request::{ Body };
use iron::modifiers::Redirect;
use router::Router;
use url::Host;

fn main() {
    let router = router!(
        get "/" => redirect_handler,
        get "*" => redirect_handler
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
