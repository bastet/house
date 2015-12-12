use time::Timespec;
use iron::typemap::Key;
use rusqlite::SqliteConnection;

#[derive(Debug, Clone, RustcDecodable)]
pub struct Register {
    pub key: String,
    pub invite: String,
    pub url: String
}

#[derive(Debug, Clone, RustcDecodable)]
pub struct Reconfigure {
    pub key: String,
    pub signature: String,
    pub payload: String
}

#[derive(Debug)]
pub struct Token {
    pub id: i64,
    pub time_created: Timespec
}

pub struct SqliteConnector;
pub struct ConnectionKey;
impl Key for ConnectionKey {
    type Value = SqliteConnection;
}
