mod local_file;
mod local_file_header;
mod signature;

#[derive(Debug)]
pub enum Error {
    DecompressError(flate2::DecompressError),
    FromUtf8(std::string::FromUtf8Error),
    IoError(std::io::Error),
    InvalidSignature,
    LocalFileHeaderParseError,
    LocalFileNotFound,
}

impl From<flate2::DecompressError> for Error {
    fn from(inner: flate2::DecompressError) -> Self {
        Error::DecompressError(inner)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(inner: std::string::FromUtf8Error) -> Self {
        Error::FromUtf8(inner)
    }
}

impl From<std::io::Error> for Error {
    fn from(inner: std::io::Error) -> Self {
        Error::IoError(inner)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub use local_file::{find_local_file, LocalFile};
pub use local_file_header::LocalFileHeader;
pub use signature::Signature;
