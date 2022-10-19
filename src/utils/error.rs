use crate::webserver::ToResponseBody;
use json::{object, JsonValue};

use std::convert::From;
use std::fs::File;
use std::path::PathBuf;
use std::{error, fmt, io};

use super::tempfile::TempFile;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

#[derive(Debug)]
pub struct PersistError<F = File> {
    pub error: io::Error,
    pub file: TempFile<F>,
}

#[derive(Debug)]
pub struct PathError {
    pub path: PathBuf,
    pub error: io::Error,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at path {:?}", self.error, self.path)
    }
}

impl error::Error for PathError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.error.source()
    }
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

impl<F> From<PersistError<F>> for Error {
    fn from(err: PersistError<F>) -> Self {
        Self {
            message: format!(
                "failed to persist temporary file, err: {:?}, file_name: {:?}",
                err.error.to_string(),
                err.file.path.as_os_str()
            ),
        }
    }
}

impl From<PathError> for Error {
    fn from(err: PathError) -> Self {
        Self {
            message: format!(
                "err: {:?}, path: {:?}",
                err.error.to_string(),
                err.path.to_str()
            ),
        }
    }
}

impl From<PathError> for io::Error {
    fn from(err: PathError) -> Self {
        io::Error::new(err.error.kind(), err)
    }
}

impl ToResponseBody for Error {
    fn to_json(&self) -> JsonValue {
        object! {
            message: self.message.clone()
        }
    }
}
