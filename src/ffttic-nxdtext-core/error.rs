use std::io;
use std::fmt;


pub enum NxdError {
    Io(io::Error),
    InvalidHeader,
    UnsupportedFormat,
}

impl From<io::Error> for NxdError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl fmt::Display for NxdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NxdError::Io(ioerr) => ioerr.fmt(f),
            NxdError::InvalidHeader => write!(f, "Invalid file header"),
            NxdError::UnsupportedFormat => write!(f, "Unsupported format"),
        }
    }
}
