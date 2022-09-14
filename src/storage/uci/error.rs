use std::fmt::{Debug, Display, Formatter};
use std::option::Option::None;

#[derive(Debug, Clone)]
pub enum Error {
    Message(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Message(_) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => Display::fmt(msg, f),
        }
    }
}