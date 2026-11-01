pub mod activity;
pub mod categories;
pub mod dns;
pub mod errors;
pub mod evidence;
pub mod keywords;
pub mod maintenance;
pub mod outbox;
pub mod records;
pub mod rules;

pub use records::*;

use crate::error::StorageError;
use crate::schema::{initialize_database, initialize_database_with_guard};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct StorageEngine {
    pub(crate) conn: Arc<Mutex<Connection>>,
}

impl StorageEngine {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let p = path.as_ref();
        let mut conn = Connection::open(p)?;
        initialize_database_with_guard(&mut conn, Some(p))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> Result<Self, StorageError> {
        let mut conn = Connection::open_in_memory()?;
        initialize_database(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}
