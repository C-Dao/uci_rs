use std::convert::From;
use std::fs::File;
use std::io::IntoInnerError;
use std::path::PathBuf;
use std::{error, fmt, io};

use crate::file::TempFile;

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

impl<W> From<IntoInnerError<W>> for Error {
    fn from(err: IntoInnerError<W>) -> Self {
        Self {
            message: err.error().to_string(),
        }
    }
}
