#![forbid(unsafe_code)]

mod binary;
mod error;
mod nxd;
mod nxd_tables;

pub use error::NxdError;
pub use nxd::{read_rows, update_rows};
