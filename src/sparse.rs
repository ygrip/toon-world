use std::collections::HashSet;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Map, Value};

const HEADER: &str = "@toon-world/sparse-v1\n";

/// Encodes a non-empty root array of objects as sparse-TOON.
///
/// `Ok(None)` means that the value is not eligible and callers should use standard TOON.
pub fn encode(value: &Value) -> Result<Option<String>> {
    let Value::Array(rows) = value else {
        return Ok(None);
    };
    if rows.is_empty() || !rows.iter().all(Value::is_object) {
        return Ok(None);
    }

    let mut columns = Vec::new();
    let mut seen = HashSet::new();
    let mut flattened_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let Value::Object(row) = row else {
            unreachable!("array members were checked as objects");
        };
        let mut flattened = Vec::new();
        flatten_object(row, "", &mut flattened);
        for (path, _) in &flattened {
            if seen.insert(path.clone()) {
                columns.push(path.clone());
            }
        }
        flattened_rows.push(flattened);
    }
    if columns.is_empty() {
        return Ok(None);
    }

    let column_line = columns
        .iter()
        .map(serde_json::to_string)
        .collect::<serde_json::Result<Vec<_>>>()?
        .join(",");
    let mut output = format!("{HEADER}[{}]{{{column_line}}}:\n", rows.len());
    for flattened in flattened_rows {
        let cells = columns
            .iter()
            .map(|column| {
                flattened
                    .iter()
                    .find(|(path, _)| path == column)
                    .map(|(_, value)| serde_json::to_string(value).map_err(Into::into))
                    .unwrap_or_else(|| Ok("~".to_owned()))
            })
            .collect::<Result<Vec<_>>>()?;
        output.push_str(&cells.join(","));
        output.push('\n');
    }
    Ok(Some(output))
}

/// Decodes sparse-TOON. `Ok(None)` means the input is standard TOON.
pub fn decode(input: &str) -> Result<Option<Value>> {
    let Some(body) = input.strip_prefix(HEADER) else {
        return Ok(None);
    };
    let (shape, row_lines) = body
        .split_once('\n')
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: missing table rows"))?;
    let (row_count, columns) = parse_shape(shape)?;
    let lines = row_lines
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.len() != row_count {
        bail!(
            "error[parse:sparse-toon]: expected {row_count} rows, found {}",
            lines.len()
        );
    }

    let mut rows = Vec::with_capacity(row_count);
    for line in lines {
        let cells = split_cells(line)?;
        if cells.len() != columns.len() {
            bail!(
                "error[parse:sparse-toon]: expected {} cells, found {}",
                columns.len(),
                cells.len()
            );
        }
        let mut row = Map::new();
        for (path, cell) in columns.iter().zip(cells) {
            if cell == "~" {
                continue;
            }
            let value = serde_json::from_str(&cell)
                .with_context(|| "error[parse:sparse-toon]: invalid JSON cell")?;
            set_pointer(&mut row, path, value)?;
        }
        rows.push(Value::Object(row));
    }
    Ok(Some(Value::Array(rows)))
}

fn flatten_object(object: &Map<String, Value>, prefix: &str, output: &mut Vec<(String, Value)>) {
    for (key, value) in object {
        let path = format!("{prefix}/{}", escape_pointer(key));
        match value {
            Value::Object(child) if !child.is_empty() => flatten_object(child, &path, output),
            _ => output.push((path, value.clone())),
        }
    }
}

fn parse_shape(shape: &str) -> Result<(usize, Vec<String>)> {
    let body = shape
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix("}:"))
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid table header"))?;
    let (count, columns) = body
        .split_once("]{")
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid table header"))?;
    let count = count
        .parse::<usize>()
        .with_context(|| "error[parse:sparse-toon]: invalid row count")?;
    let columns = serde_json::from_str::<Vec<String>>(&format!("[{columns}]"))
        .with_context(|| "error[parse:sparse-toon]: invalid column list")?;
    if columns.is_empty() || columns.iter().any(|column| !column.starts_with('/')) {
        bail!("error[parse:sparse-toon]: columns must be JSON Pointer paths");
    }
    Ok((count, columns))
}

fn split_cells(line: &str) -> Result<Vec<String>> {
    let mut cells = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '[' | '{' => depth += 1,
            ']' | '}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| anyhow!("error[parse:sparse-toon]: unbalanced cell"))?;
            }
            ',' if depth == 0 => {
                cells.push(line[start..index].to_owned());
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if quoted || depth != 0 {
        bail!("error[parse:sparse-toon]: unbalanced cell");
    }
    cells.push(line[start..].to_owned());
    Ok(cells)
}

fn set_pointer(object: &mut Map<String, Value>, pointer: &str, value: Value) -> Result<()> {
    let segments = pointer
        .strip_prefix('/')
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid column path"))?
        .split('/')
        .map(unescape_pointer)
        .collect::<Result<Vec<_>>>()?;
    let (last, parents) = segments
        .split_last()
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid column path"))?;
    let mut current = object;
    for segment in parents {
        let entry = current
            .entry(segment.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let Value::Object(next) = entry else {
            bail!("error[parse:sparse-toon]: conflicting column paths");
        };
        current = next;
    }
    if current.insert(last.clone(), value).is_some() {
        bail!("error[parse:sparse-toon]: duplicate column path");
    }
    Ok(())
}

fn escape_pointer(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

fn unescape_pointer(segment: &str) -> Result<String> {
    let mut output = String::new();
    let mut characters = segment.chars();
    while let Some(character) = characters.next() {
        if character != '~' {
            output.push(character);
            continue;
        }
        match characters.next() {
            Some('0') => output.push('~'),
            Some('1') => output.push('/'),
            _ => bail!("error[parse:sparse-toon]: invalid JSON Pointer escape"),
        }
    }
    Ok(output)
}
