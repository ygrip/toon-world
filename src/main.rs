use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use toon_world::cli::Args;
use toon_world::{input, output, query};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let input = input::read_json(args.file.as_deref())?;
    let values = query::execute(&args.query, input)?;
    let rendered = output::encode_results(&values, args.to)?;

    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(rendered.as_bytes())?;
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        stdout.write_all(b"\n")?;
    }

    Ok(())
}
