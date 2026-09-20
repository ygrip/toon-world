use anyhow::{bail, Context, Result};
use jaq_json::Val;
use serde_json::Value;

use crate::cli::OutputFormat;
use crate::sparse;

pub fn encode_results(values: &[Val], format: OutputFormat) -> Result<String> {
    encode_results_with_options(values, format, false)
}

pub fn encode_results_with_options(
    values: &[Val],
    format: OutputFormat,
    compact: bool,
) -> Result<String> {
    if compact && format != OutputFormat::Toon {
        bail!("error[encode:sparse-toon]: --compact requires --to toon");
    }
    match format {
        OutputFormat::Text => encode_text_results(values),
        OutputFormat::Json => {
            let value = structured_result(values)?;
            serde_json::to_string(&value).context("error[encode:json]: could not encode result")
        }
        OutputFormat::Toon => {
            let value = structured_result(values)?;
            let standard = toon_format::encode_default(&value)
                .map_err(|error| anyhow::anyhow!("error[encode:toon]: {error}"))?;
            if compact {
                if let Some(rendered) = sparse::encode(&value)? {
                    if sparse::token_count(&rendered)? < sparse::token_count(&standard)? {
                        return Ok(rendered);
                    }
                }
            }
            Ok(standard)
        }
    }
}

fn structured_result(values: &[Val]) -> Result<Value> {
    match values {
        [value] => to_json_value(value),
        _ => values
            .iter()
            .map(to_json_value)
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
    }
}

fn to_json_value(value: &Val) -> Result<Value> {
    let rendered = value.to_string();
    serde_json::from_str(&rendered).map_err(|error| {
        anyhow::anyhow!("error[normalize]: query result is not JSON-compatible: {error}")
    })
}

fn encode_text_results(values: &[Val]) -> Result<String> {
    values
        .iter()
        .map(encode_text_value)
        .collect::<Result<Vec<_>>>()
        .map(|parts| parts.join("\n"))
}

fn encode_text_value(value: &Val) -> Result<String> {
    match value {
        Val::Null => Ok("null".to_owned()),
        Val::Bool(value) => Ok(value.to_string()),
        Val::Num(value) => Ok(value.to_string()),
        Val::TStr(value) => Ok(String::from_utf8_lossy(value.as_ref()).into_owned()),
        Val::BStr(_) => bail!("error[encode:text]: binary strings are not supported"),
        Val::Arr(_) | Val::Obj(_) => {
            bail!("error[encode:text]: text output requires scalar query results")
        }
    }
}
