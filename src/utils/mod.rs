pub mod error;
pub mod tempfile;

pub use error::Error;
pub type Result<T, E = Error> = core::result::Result<T, E>;
