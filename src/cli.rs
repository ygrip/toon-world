use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum InputFormat {
    Json,
    Ndjson,
    Csv,
    Yaml,
    Toml,
    Xml,
    Toon,
    Markdown,
    Html,
}

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
    about = "Query structured and document data and emit compact TOON"
)]
pub struct Args {
    /// Input file. Reads stdin when omitted or when FILE is '-'.
    #[arg(conflicts_with = "data")]
    pub file: Option<PathBuf>,

    /// Raw input data. Mutually exclusive with FILE.
    #[arg(long, conflicts_with = "file")]
    pub data: Option<String>,

    /// Override input format. Otherwise inferred from a known extension, falling back to JSON.
    #[arg(long, value_enum)]
    pub from: Option<InputFormat>,

    /// Suppress non-fatal warnings.
    #[arg(long, conflicts_with = "warnings_as_errors")]
    pub quiet: bool,

    /// Treat any warning as an error and exit non-zero.
    #[arg(long, conflicts_with = "quiet")]
    pub warnings_as_errors: bool,

    /// Emit a compact content-oriented document model where supported (currently HTML).
    #[arg(long)]
    pub semantic: bool,

    /// jq-compatible query. Defaults to the identity filter.
    #[arg(short = 'q', long, default_value = ".")]
    pub query: String,

    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Toon)]
    pub to: OutputFormat,
}
