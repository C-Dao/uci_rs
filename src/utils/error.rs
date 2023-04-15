use std::convert::From;
#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new<T>(message: T) -> Error
    where
        T: Into<String>,
    {
        Error {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}
