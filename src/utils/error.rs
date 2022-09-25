use crate::webserver::ToResponseBody;
use json::{object, JsonValue};
use std::convert::From;
use tempfile::PersistError;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: String) -> Error {
        Error { message }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<PersistError> for Error {
    fn from(err: PersistError) -> Self {
        Self {
            message: format!(
                "err: {:?}, file_name: {:?}",
                err.error.to_string(),
                err.file.path().to_str()
            ),
        }
    }
}

impl ToResponseBody for Error {
    fn to_json(&self) -> JsonValue {
        object! {
            message: self.message.clone()
        }
    }
}
