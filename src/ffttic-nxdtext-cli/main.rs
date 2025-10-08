// Copyright (C) 2025  Mátyás Mustoha
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]

mod cli;
mod error;
mod export;
mod import;

use crate::{
    cli::{Cli, CliCommand},
    error::Error,
};
use clap::Parser;
use std::path::Path;


fn path_to_tablename(path: &Path) -> Result<&str, Error> {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.split_once('.').map(|(first, _)| first).unwrap_or(s))
        .ok_or(Error(
            "Could not determine the table name from the file name".to_owned(),
        ))
}


fn inner_main(args: Cli) -> Result<(), Error> {
    match &args.command {
        CliCommand::Export { nxd, output } => {
            export::run(nxd, &output.out_json, &output.out_po)?;
        },
        CliCommand::Import { nxd, input, out } => {
            import::run(nxd, &input.json, &input.po, out)?;
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
