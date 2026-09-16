use anyhow::{Result, bail};
use jaq_json::Val;

use crate::cli::OutputFormat;

pub fn encode_results(values: &[Val], format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Text => encode_text_results(values),
        OutputFormat::Json => {
            let value = structured_result(values);
            Ok(serde_json::to_string(&value)?)
        }
        OutputFormat::Toon => {
            let value = structured_result(values);
            toon_format::encode_default(&value)
                .map_err(|error| anyhow::anyhow!("error[encode:toon]: {error}"))
        }
    }
}

fn structured_result(values: &[Val]) -> Val {
    match values {
        [value] => value.clone(),
        _ => values.iter().cloned().collect(),
    }
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
        Val::TStr(value) => Ok(String::from_utf8_lossy(value.as_ref().as_ref()).into_owned()),
        Val::BStr(_) => bail!("error[encode:text]: binary strings are not supported"),
        Val::Arr(_) | Val::Obj(_) => {
            bail!("error[encode:text]: text output requires scalar query results")
        }
    }
}
