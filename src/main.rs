use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use toon_world::cli::Args;
use toon_world::{diagnostics, input, output, query, stats};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let input = input::read_with_fallback(
        args.file.as_deref(),
        args.from,
        args.data.as_deref(),
        args.semantic,
        args.fallback,
    )?;

    match input {
        input::ReadOutcome::Parsed(input) => {
            let rendered_warnings = diagnostics::resolve_warnings(
                &input.warnings,
                args.quiet,
                args.warnings_as_errors,
            )?;
            if !rendered_warnings.is_empty() {
                eprintln!("{rendered_warnings}");
            }
            let input_bytes = input.byte_len;
            let values = query::execute(&args.query, input.value)?;
            let rendered = output::encode_results_with_options(&values, args.to, args.compact)?;

            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(rendered.as_bytes())?;
            if !rendered.is_empty() && !rendered.ends_with('\n') {
                stdout.write_all(b"\n")?;
            }
            if args.stats {
                eprintln!("{}", stats::render(input_bytes, rendered.len()));
            }
        }
        input::ReadOutcome::Fallback(fallback) => {
            let rendered_warnings = diagnostics::resolve_warnings(
                &fallback.warnings,
                args.quiet,
                args.warnings_as_errors,
            )?;
            if !rendered_warnings.is_empty() {
                eprintln!("{rendered_warnings}");
            }

            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            stdout.write_all(&fallback.bytes)?;
            if args.stats {
                eprintln!(
                    "{}",
                    stats::render(fallback.bytes.len(), fallback.bytes.len())
                );
            }
        }
    }

    Ok(())
}
