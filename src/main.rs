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
use iron::{ Url, status };
use iron::middleware::{ BeforeMiddleware };
use rusqlite::{ SqliteConnection, SqliteResult };

mod models;
use models::{ ConnectionKey, SqliteConnector, prepare_database };

mod handlers;
use handlers::{ token_handler, redirect_handler, register_handler, reconfigure_handler };

fn open_db_connection() -> SqliteResult<SqliteConnection> {
    SqliteConnection::open("bastet.db")
}

fn main() {
    // Open new sqlite connection with a flag to allow multi-threading
    let conn = prepare_database(open_db_connection().expect("Failed to open db connection (main)"));

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

impl BeforeMiddleware for SqliteConnector {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        match open_db_connection() {
            Ok(conn) => {
                req.extensions.insert::<ConnectionKey>(conn);
                Ok(())
            },
            Err(err) => Err(IronError::new(err, (status::InternalServerError, "Database Connection Failed")))
        }
    }
}
