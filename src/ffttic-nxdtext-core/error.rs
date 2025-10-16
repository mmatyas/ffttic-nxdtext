// Copyright (C) 2025  Mátyás Mustoha

use std::{fmt, io};


pub enum NxdError {
    Io(io::Error),
    InvalidHeader,
    UnsupportedFormat,
    Utf8Error {
        offset: u64,
    },

    RowContext {
        row: usize,
        source: Box<NxdError>,
    },
    CellContext {
        col: usize,
        offset: u64,
        source: Box<NxdError>,
    },
}

impl From<io::Error> for NxdError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl fmt::Display for NxdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NxdError::Io(ioerr) => write!(f, "I/O error: {}", ioerr),
            NxdError::InvalidHeader => write!(f, "Invalid file header"),
            NxdError::UnsupportedFormat => write!(f, "Unsupported format"),
            NxdError::Utf8Error { offset } => {
                write!(f, "The text that starts at offset {} is not a valid UTF-8 sequence", offset)
            },
            NxdError::RowContext { row, source } => {
                write!(f, "Error when trying to read row {}:\n  {}", row, source)
            },
            NxdError::CellContext { col, offset, source } => {
                write!(f, "Error when trying to read cell {} at offset {}:\n    {}", col, offset, source)
            },
        }
    }
}
