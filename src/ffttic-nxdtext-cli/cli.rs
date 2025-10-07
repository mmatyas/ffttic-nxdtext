use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;


/// A tool for extracting and injecting text to/from NXD files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Export text from NXD files.
    Export {
        /// The source NXD file
        nxd: PathBuf,

        #[command(flatten)]
        output: CliExportOutput,
    },
    /// Import text from either a JSON or a PO file.
    Import {
        /// The source NXD file
        nxd: PathBuf,

        #[command(flatten)]
        input: CliInjectInput,

        /// The output NXD file
        #[arg(short, long, value_name = "FILE", required = true)]
        out: PathBuf,
    },
}

#[derive(Args, Debug)]
#[group(required = true, multiple = true)]
pub struct CliExportOutput {
    /// The output JSON file
    #[arg(long, value_name = "FILE")]
    pub out_json: Option<PathBuf>,

    /// The output PO file
    #[arg(long, value_name = "FILE")]
    pub out_po: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct CliInjectInput {
    /// The input JSON file
    #[arg(long, value_name = "FILE")]
    pub json: Option<PathBuf>,

    /// The input PO file
    #[arg(long, value_name = "FILE")]
    pub po: Option<PathBuf>,
}
