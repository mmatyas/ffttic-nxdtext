#![forbid(unsafe_code)]

mod cli;
mod error;
mod dump;

use crate::cli::{Cli, CliCommand};
use crate::error::Error;
use clap::Parser;
use std::path::Path;


fn path_to_tablename(path: &Path) -> Result<&str, Error> {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.split_once('.').map(|(first, _)| first).unwrap_or(s))
        .ok_or(Error("Could not determine the table name from the file name".to_owned()))
}


fn inner_main(args: Cli) -> Result<(), Error> {
    match &args.command {
        CliCommand::Dump { nxd, output } => {
            dump::run(nxd, &output.out_json, &output.out_po)?;
        },
        CliCommand::Insert { nxd, input, out } => {
            println!("inject {:#?} {:#?} {:#?}", nxd, input, out);
        },
    }
    Ok(())
}


fn main() {
    let args = Cli::parse();
    if let Err(msg) = inner_main(args) {
        eprintln!("{}", msg.0);
        std::process::exit(1);
    }
}
