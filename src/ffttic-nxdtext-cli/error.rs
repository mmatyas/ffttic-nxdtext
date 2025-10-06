use ffttic_nxdtext_core::NxdError;
use std::io;


#[derive(Debug)]
pub struct Error(pub String);

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error(err.to_string())
    }
}

impl From<NxdError> for Error {
    fn from(err: NxdError) -> Self {
        match err {
            NxdError::Io(ioerr) => Self(ioerr.to_string()),
            NxdError::InvalidHeader => Self("Invalid file header".to_owned()),
            NxdError::UnsupportedFormat => Self("Unsupported format".to_owned()),
        }
    }
}
