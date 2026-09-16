use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Toon,
    Json,
    Text,
}

#[derive(Debug, Parser)]
#[command(
    name = "toon-world",
    version,
    about = "Query structured data and emit compact TOON"
)]
pub struct Args {
    /// Input file. Reads stdin when omitted or when FILE is '-'.
    pub file: Option<PathBuf>,

    /// jq-compatible query. Defaults to the identity filter.
    #[arg(short = 'q', long, default_value = ".")]
    pub query: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Toon)]
    pub to: OutputFormat,
}
