use std::fmt;


#[derive(Debug)]
pub struct Error(pub String);

impl<E: fmt::Display> From<E> for Error {
    fn from(err: E) -> Self {
        Error(err.to_string())
    }
}
