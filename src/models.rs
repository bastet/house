use time::Timespec;
use iron::typemap::Key;
use rusqlite::SqliteConnection;

#[derive(Debug, Clone, RustcDecodable)]
pub struct Register {

    /// public key of this hub (will be used to control reconfigure this redirect)
    pub key: String,

    /// Invite code to consume
    pub invite: String,

    /// URL to redirect to
    pub url: String
}

#[derive(Debug, Clone, RustcDecodable)]
pub struct Reconfigure {
    pub key: String,
    pub signature: String,
    pub payload: String
}

#[derive(Clone)]
pub struct IpMapping {
    pub ip: String,
    pub port: i32
}

pub struct SqliteConnector;
pub struct ConnectionKey;
impl Key for ConnectionKey {
    type Value = SqliteConnection;
}

pub fn prepare_database(conn: SqliteConnection) -> SqliteConnection {
    //Create tokens table
    conn.execute("CREATE TABLE IF NOT EXISTS tokens (
        id              INTEGER PRIMARY KEY,
        time_created    INTEGER
    )", &[]).expect("Failed to create tokens table");

    //Create redirect table
    conn.execute("CREATE TABLE IF NOT EXISTS redirects (
        id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        public_ip       STRING,
        time_created    INTEGER,
        internal_ip     String,
        internal_port   INTEGER
    )", &[]).expect("Failed to create redirects table");

    //Create users table
    conn.execute("CREATE TABLE IF NOT EXISTS users (
        id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
        time_created    TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
        public_key      BLOB
    )", &[]).expect("Failed to create users table");

    conn
}
