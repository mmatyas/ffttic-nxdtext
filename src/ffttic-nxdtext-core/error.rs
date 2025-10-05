use std::io;

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
