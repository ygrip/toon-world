use std::fmt;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    category: String,
    message: String,
}

impl Warning {
    pub fn new(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "warning[{}]: {}", self.category, self.message)
    }
}

pub fn resolve_warnings(
    warnings: &[Warning],
    quiet: bool,
    warnings_as_errors: bool,
) -> Result<String> {
    if warnings.is_empty() || quiet {
        return Ok(String::new());
    }

    let rendered = warnings
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    if warnings_as_errors {
        bail!("error[warning-as-error]: {rendered}");
    }

    Ok(rendered)
}
