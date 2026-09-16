use std::fs::File;
use std::io::{stdin, Read};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use jaq_json::Val;

pub fn read_json(path: Option<&Path>) -> Result<Val> {
    let mut bytes = Vec::new();

    match path {
        Some(path) if path != Path::new("-") => {
            File::open(path)
                .with_context(|| format!("error[input]: could not open {}", path.display()))?
                .read_to_end(&mut bytes)
                .with_context(|| format!("error[input]: could not read {}", path.display()))?;
        }
        _ => {
            stdin()
                .lock()
                .read_to_end(&mut bytes)
                .context("error[input]: could not read stdin")?;
        }
    }

    jaq_json::read::parse_single(&bytes)
        .map_err(|err| anyhow!("error[parse:json]: {err:?}"))
}
