use std::collections::HashSet;
use std::fs::File;
use std::io::{stdin, Read};
use std::path::Path;
use std::str;

use anyhow::{anyhow, bail, Context, Result};
use jaq_json::Val;

use crate::cli::InputFormat;
use crate::diagnostics::Warning;
use crate::markdown;

pub struct ReadResult {
    pub value: Val,
    pub warnings: Vec<Warning>,
}

pub fn read(
    path: Option<&Path>,
    explicit_format: Option<InputFormat>,
    data: Option<&str>,
) -> Result<ReadResult> {
    let (format, warnings) = resolve_format(path, explicit_format);
    let value = if let Some(data) = data {
        parse_bytes(data.as_bytes(), format)?
    } else {
        let bytes = read_bytes(path)?;
        parse_bytes(&bytes, format)?
    };
    Ok(ReadResult { value, warnings })
}

fn resolve_format(
    path: Option<&Path>,
    explicit_format: Option<InputFormat>,
) -> (InputFormat, Vec<Warning>) {
    if let Some(format) = explicit_format {
        return (format, Vec::new());
    }
    let Some(path) = path.filter(|path| *path != Path::new("-")) else {
        return (InputFormat::Json, Vec::new());
    };
    if let Some(format) = detect_format(path) {
        return (format, Vec::new());
    }
    let reason = path
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .map(|extension| format!("unknown extension '.{extension}'"))
        .unwrap_or_else(|| "could not infer format from filename".to_owned());
    (
        InputFormat::Json,
        vec![Warning::new("input", format!("{reason}; assuming JSON"))],
    )
}

pub fn detect_format(path: &Path) -> Option<InputFormat> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "json" => Some(InputFormat::Json),
        "jsonl" | "ndjson" => Some(InputFormat::Ndjson),
        "csv" => Some(InputFormat::Csv),
        "yaml" | "yml" => Some(InputFormat::Yaml),
        "toml" => Some(InputFormat::Toml),
        "xml" | "xhtml" => Some(InputFormat::Xml),
        "toon" => Some(InputFormat::Toon),
        "md" | "markdown" => Some(InputFormat::Markdown),
        _ => None,
    }
}

pub fn parse_bytes(bytes: &[u8], format: InputFormat) -> Result<Val> {
    match format {
        InputFormat::Json => jaq_json::read::parse_single(bytes)
            .map_err(|error| anyhow!("error[parse:json]: {error:?}")),
        InputFormat::Ndjson => parse_ndjson(bytes),
        InputFormat::Csv => parse_csv(bytes),
        InputFormat::Yaml => parse_yaml(as_utf8(bytes, "yaml")?),
        InputFormat::Toml => jaq_fmts::read::toml::parse(as_utf8(bytes, "toml")?)
            .map_err(|error| anyhow!("error[parse:toml]: {error}")),
        InputFormat::Xml => parse_xml(as_utf8(bytes, "xml")?),
        InputFormat::Toon => parse_toon(as_utf8(bytes, "toon")?),
        InputFormat::Markdown => markdown::parse(as_utf8(bytes, "markdown")?),
    }
}

fn read_bytes(path: Option<&Path>) -> Result<Vec<u8>> {
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
    Ok(bytes)
}

fn parse_ndjson(bytes: &[u8]) -> Result<Val> {
    let values = jaq_json::read::parse_many(bytes)
        .map(|value| value.map_err(|error| anyhow!("error[parse:ndjson]: {error:?}")))
        .collect::<Result<Vec<_>>>()?;
    Ok(values.into_iter().collect())
}

fn parse_csv(bytes: &[u8]) -> Result<Val> {
    let mut reader = csv::ReaderBuilder::new().from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|error| anyhow!("error[parse:csv]: {error}"))?
        .clone();
    let mut seen = HashSet::new();
    for header in headers.iter() {
        if header.is_empty() {
            bail!("error[parse:csv]: header names must not be empty");
        }
        if !seen.insert(header.to_owned()) {
            bail!("error[parse:csv]: duplicate header '{header}'");
        }
    }
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| anyhow!("error[parse:csv]: {error}"))?;
        let fields = headers
            .iter()
            .zip(record.iter())
            .map(|(header, cell)| (header.to_owned().into(), cell.to_owned().into()));
        rows.push(Val::obj(fields.collect()));
    }
    Ok(rows.into_iter().collect())
}

fn parse_yaml(input: &str) -> Result<Val> {
    let values = jaq_fmts::read::yaml::parse_many(input)
        .map(|value| value.map_err(|error| anyhow!("error[parse:yaml]: {error}")))
        .collect::<Result<Vec<_>>>()?;
    Ok(collapse_single(values))
}

fn parse_xml(input: &str) -> Result<Val> {
    let values = jaq_fmts::read::xml::parse_many(input)
        .map(|value| value.map_err(|error| anyhow!("error[parse:xml]: {error}")))
        .collect::<Result<Vec<_>>>();

    match values {
        Ok(values) => Ok(collapse_single(values)),
        Err(_) => parse_xml_fragment(input),
    }
}

fn parse_xml_fragment(input: &str) -> Result<Val> {
    let wrapped = format!("<toon-world-fragment>{input}</toon-world-fragment>");
    let wrapper = jaq_fmts::read::xml::parse_many(&wrapped)
        .next()
        .expect("synthetic XML wrapper has one root")
        .map_err(|error| anyhow!("error[parse:xml]: {error}"))?;

    let Val::Obj(wrapper) = wrapper else {
        bail!("error[parse:xml]: synthetic wrapper did not normalize to an object");
    };
    let children = wrapper
        .get(&Val::utf8_str("c"))
        .cloned()
        .unwrap_or_else(|| Val::Arr(Default::default()));

    let Val::Arr(children) = children else {
        bail!("error[parse:xml]: synthetic wrapper did not contain child nodes");
    };

    Ok(children.iter().cloned().collect())
}

fn parse_toon(input: &str) -> Result<Val> {
    let value: serde_json::Value = toon_format::decode_default(input)
        .map_err(|error| anyhow!("error[parse:toon]: {error}"))?;
    serde_json::from_value(value)
        .map_err(|error| anyhow!("error[parse:toon]: could not normalize decoded value: {error}"))
}

fn collapse_single(values: Vec<Val>) -> Val {
    if values.len() == 1 {
        values.into_iter().next().expect("length checked")
    } else {
        values.into_iter().collect()
    }
}

fn as_utf8<'a>(bytes: &'a [u8], format: &str) -> Result<&'a str> {
    str::from_utf8(bytes)
        .with_context(|| format!("error[parse:{format}]: input must be valid UTF-8"))
}
