#![forbid(unsafe_code)]

mod cli;

use crate::cli::{Cli, CliCommand};
use ffttic_nxdtext_core::{
    NxdError,
    nxd,
    NXD_COLUMNS,
};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};


fn nxderr_to_str(err: NxdError) -> String {
    match err {
        NxdError::Io(ioerr) => ioerr.to_string(),
        NxdError::InvalidHeader => "Invalid file header".to_owned(),
        NxdError::UnsupportedFormat => "Unsupported format".to_owned(),
    }
}

fn ioerr_to_str(err: io::Error) -> String {
    err.to_string()
}


fn path_to_tablename(path: &Path) -> Result<&str, &'static str> {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.split_once('.').map(|(first, _)| first).unwrap_or(s))
        .ok_or("Could not determine the table name from the file name")
}


fn dump(
    nxd_path: &Path,
    out_json: &Option<PathBuf>,
    out_po: &Option<PathBuf>,
) -> Result<(), String> {
    let tablename = path_to_tablename(&nxd_path)?;
    let row_definition = NXD_COLUMNS
        .get(tablename)
        .ok_or(format!("Unknown or unsupported table name `{}`", tablename))?;

    let nxdfile = File::open(nxd_path)
        .map_err(|e| format!("Could not open input file: {}", e))?;
    let mut reader = BufReader::new(nxdfile);

    let rows = nxd::Header::read_rowinfos(&mut reader)
        .map_err(nxderr_to_str)?
        .into_iter()
        .map(|rowinfo| nxd::read_row(&mut reader, &row_definition, &rowinfo))
        .collect::<Result<Vec<_>, _>>()
        .map_err(nxderr_to_str)?
        .into_iter()
        .enumerate()
        .flat_map(|(row_idx, row)| row
            .into_iter()
            .map(move |(cell_idx, text)| (row_idx, cell_idx, text))
        )
        .collect::<Vec<_>>();

    if let Some(json_path) = out_json {
        let mut map = serde_json::Map::with_capacity(rows.len());

        for (row_idx, cell_idx, text) in rows {
            let key = format!("{}/{}/{}", tablename, row_idx, cell_idx);
            map.insert(key, serde_json::Value::String(text));
        }

        let json_content = serde_json::to_string_pretty(&map)
            .map_err(|err| err.to_string())?;

        let mut json_file = File::create(json_path).map_err(ioerr_to_str)?;
        json_file.write_all(json_content.as_bytes()).map_err(ioerr_to_str)?;
    }
    Ok(())
}


fn inner_main(args: Cli) -> Result<(), String> {
    match &args.command {
        CliCommand::Dump { nxd, output } => {
            dump(nxd, &output.out_json, &output.out_po)?;
        },
        CliCommand::Inject { nxd, input, out } => {
            println!("inject {:#?} {:#?} {:#?}", nxd, input, out);
        },
    }
    Ok(())
}


fn main() {
    let args = Cli::parse();
    if let Err(msg) = inner_main(args) {
        eprintln!("{}", msg);
        std::process::exit(1);
    }
}
