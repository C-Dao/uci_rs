mod error;

pub use error::Error;
pub use error::PathError;
pub use error::PersistError;
pub type Result<T> = std::result::Result<T, Error>;
