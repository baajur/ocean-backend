use crate::db;
use std::error::Error;

pub mod mandela;
pub mod user;

pub type RequestResult = Result<Option<serde_json::Value>, Box<dyn Error>>;
pub type RequestHandler = fn(RequestData) -> RequestResult;

pub struct RequestData {
    db: db::Db,
    params: Option<serde_json::Value>,
}

impl RequestData {
    pub fn new(db: db::Db, params: Option<serde_json::Value>) -> Self {
        Self { db, params }
    }
}
