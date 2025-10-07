#![forbid(unsafe_code)]

mod binary;
mod error;
mod nxd_tables;
mod nxd;

pub use error::NxdError;
pub use nxd::{
    read_rows,
    update_rows,
};
