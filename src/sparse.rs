use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Map, Value};

const HEADER: &str = "@toon-world/sparse-v1\n";

#[derive(Clone)]
enum Cell {
    Value(Value),
    Ref(usize),
    Missing,
}
#[derive(Clone)]
enum Node {
    Object(Vec<(String, Cell)>),
    List(Vec<Cell>),
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<Cell>>,
    },
}
struct Builder {
    nodes: Vec<Option<Node>>,
}

impl Builder {
    fn value(&mut self, value: &Value) -> Cell {
        if !value.is_array() && !value.is_object() {
            return Cell::Value(value.clone());
        }
        let id = self.nodes.len();
        self.nodes.push(None);
        let node = match value {
            Value::Object(object) => Node::Object(
                object
                    .iter()
                    .map(|(key, value)| (key.clone(), self.value(value)))
                    .collect(),
            ),
            Value::Array(values) if !values.is_empty() && values.iter().all(Value::is_object) => {
                self.table(values)
            }
            Value::Array(values) => {
                Node::List(values.iter().map(|value| self.value(value)).collect())
            }
            _ => unreachable!(),
        };
        self.nodes[id] = Some(node);
        Cell::Ref(id)
    }
    fn table(&mut self, values: &[Value]) -> Node {
        let mut columns = Vec::new();
        let mut rows = Vec::with_capacity(values.len());
        for value in values {
            let Value::Object(object) = value else {
                unreachable!()
            };
            let mut flat = Vec::new();
            self.flatten(object, "", &mut flat);
            for (path, _) in &flat {
                if !columns.contains(path) {
                    columns.push(path.clone());
                }
            }
            rows.push(flat);
        }
        if columns.is_empty() {
            return Node::List(values.iter().map(|value| self.value(value)).collect());
        }
        let rows = rows
            .into_iter()
            .map(|flat| {
                columns
                    .iter()
                    .map(|column| {
                        flat.iter()
                            .find(|(path, _)| path == column)
                            .map(|(_, cell)| cell.clone())
                            .unwrap_or(Cell::Missing)
                    })
                    .collect()
            })
            .collect();
        Node::Table { columns, rows }
    }
    fn flatten(
        &mut self,
        object: &Map<String, Value>,
        prefix: &str,
        output: &mut Vec<(String, Cell)>,
    ) {
        for (key, value) in object {
            let path = format!("{prefix}/{}", escape_pointer(key));
            match value {
                Value::Object(child) if !child.is_empty() => self.flatten(child, &path, output),
                _ => output.push((path, self.value(value))),
            }
        }
    }
}

/// Encodes any object or array as recursive sparse-TOON v1. Scalars use standard TOON.
pub fn encode(value: &Value) -> Result<Option<String>> {
    if !value.is_array() && !value.is_object() {
        return Ok(None);
    }
    let mut builder = Builder { nodes: Vec::new() };
    let Cell::Ref(root) = builder.value(value) else {
        unreachable!()
    };
    let mut output = format!("{HEADER}root=^{root}\n");
    for (id, node) in builder.nodes.into_iter().enumerate() {
        output.push_str(&format!("^{id}="));
        render_node(node.expect("builder fills nodes"), &mut output)?;
    }
    Ok(Some(output))
}
pub fn token_count(value: &str) -> Result<usize> {
    Ok(tiktoken_rs::cl100k_base()
        .context("error[encode:sparse-toon]: could not load cl100k tokenizer")?
        .encode_with_special_tokens(value)
        .len())
}
fn render_node(node: Node, output: &mut String) -> Result<()> {
    match node {
        Node::Object(entries) => {
            output.push('{');
            for (index, (key, cell)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(&key)?);
                output.push(':');
                render_cell(cell, output)?;
            }
            output.push_str("}\n");
        }
        Node::List(cells) => {
            output.push('[');
            for (index, cell) in cells.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                render_cell(cell, output)?;
            }
            output.push_str("]\n");
        }
        Node::Table { columns, rows } => {
            output.push_str(&format!(
                "[{}]{{{}}}:\n",
                rows.len(),
                columns
                    .iter()
                    .map(serde_json::to_string)
                    .collect::<serde_json::Result<Vec<_>>>()?
                    .join(",")
            ));
            for row in rows {
                for (index, cell) in row.into_iter().enumerate() {
                    if index > 0 {
                        output.push(',');
                    }
                    render_cell(cell, output)?;
                }
                output.push('\n');
            }
        }
    };
    Ok(())
}
fn render_cell(cell: Cell, output: &mut String) -> Result<()> {
    match cell {
        Cell::Value(value) => output.push_str(&serde_json::to_string(&value)?),
        Cell::Ref(id) => output.push_str(&format!("^{id}")),
        Cell::Missing => output.push('~'),
    };
    Ok(())
}

/// Decodes current and recursive sparse-v1. `None` means standard TOON.
pub fn decode(input: &str) -> Result<Option<Value>> {
    if let Some(body) = input.strip_prefix(HEADER) {
        return if body.starts_with("root=^") {
            decode_recursive_v1(body).map(Some)
        } else {
            decode_legacy_v1(body).map(Some)
        };
    }
    Ok(None)
}
fn decode_recursive_v1(body: &str) -> Result<Value> {
    let (root, mut rest) = body
        .split_once('\n')
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: missing recursive root"))?;
    let root = parse_reference(
        root.strip_prefix("root=")
            .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid recursive root"))?,
    )?;
    let mut nodes = HashMap::new();
    while !rest.is_empty() {
        let (id, after_id) = parse_definition(rest)?;
        let (node, after_node) = parse_node(after_id)?;
        if nodes.insert(id, node).is_some() {
            bail!("error[parse:sparse-toon]: duplicate recursive node");
        }
        rest = after_node;
    }
    let mut visiting = HashSet::new();
    resolve_node(root, &nodes, &mut visiting)
}
fn parse_definition(input: &str) -> Result<(usize, &str)> {
    let body = input
        .strip_prefix('^')
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: expected recursive node"))?;
    let (id, rest) = body
        .split_once('=')
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid recursive node"))?;
    Ok((
        id.parse()
            .context("error[parse:sparse-toon]: invalid recursive node id")?,
        rest,
    ))
}
fn parse_node(input: &str) -> Result<(Node, &str)> {
    if input.starts_with('[') {
        let line_end = input
            .find('\n')
            .ok_or_else(|| anyhow!("error[parse:sparse-toon]: missing recursive node line"))?;
        if let Ok((count, columns)) = parse_shape(&input[..line_end]) {
            let mut rest = &input[line_end + 1..];
            let mut rows = Vec::with_capacity(count);
            for _ in 0..count {
                let (line, next) = rest.split_once('\n').ok_or_else(|| {
                    anyhow!("error[parse:sparse-toon]: missing recursive table row")
                })?;
                let cells = split_cells(line)?
                    .into_iter()
                    .map(|cell| parse_cell(&cell))
                    .collect::<Result<Vec<_>>>()?;
                if cells.len() != columns.len() {
                    bail!("error[parse:sparse-toon]: invalid recursive table width");
                }
                rows.push(cells);
                rest = next;
            }
            return Ok((Node::Table { columns, rows }, rest));
        }
        let (body, rest) = take_balanced(input, '[', ']')?;
        return Ok((
            Node::List(if body.is_empty() {
                Vec::new()
            } else {
                split_cells(body)?
                    .into_iter()
                    .map(|cell| parse_cell(&cell))
                    .collect::<Result<Vec<_>>>()?
            }),
            rest.strip_prefix('\n').unwrap_or(rest),
        ));
    }
    let (body, rest) = take_balanced(input, '{', '}')?;
    let mut entries = Vec::new();
    if !body.is_empty() {
        for item in split_cells(body)? {
            let (key, cell) = split_pair(&item)?;
            entries.push((
                serde_json::from_str(key)
                    .context("error[parse:sparse-toon]: invalid recursive object key")?,
                parse_cell(cell)?,
            ));
        }
    }
    Ok((
        Node::Object(entries),
        rest.strip_prefix('\n').unwrap_or(rest),
    ))
}
fn take_balanced(input: &str, open: char, close: char) -> Result<(&str, &str)> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if quoted {
            if escaped {
                escaped = false
            } else if character == '\\' {
                escaped = true
            } else if character == '"' {
                quoted = false
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    anyhow!("error[parse:sparse-toon]: unbalanced recursive node")
                })?;
                if depth == 0 {
                    return Ok((&input[1..index], &input[index + 1..]));
                }
            }
            _ => {}
        }
    }
    bail!("error[parse:sparse-toon]: unbalanced recursive node")
}
fn parse_cell(cell: &str) -> Result<Cell> {
    if cell == "~" {
        return Ok(Cell::Missing);
    }
    if cell.starts_with('^') {
        return Ok(Cell::Ref(parse_reference(cell)?));
    }
    Ok(Cell::Value(
        serde_json::from_str(cell).context("error[parse:sparse-toon]: invalid JSON cell")?,
    ))
}
fn parse_reference(value: &str) -> Result<usize> {
    value
        .strip_prefix('^')
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid recursive reference"))?
        .parse()
        .context("error[parse:sparse-toon]: invalid recursive reference")
}
fn split_pair(value: &str) -> Result<(&str, &str)> {
    let mut quoted = false;
    let mut escaped = false;
    let mut depth = 0usize;
    for (index, c) in value.char_indices() {
        if quoted {
            if escaped {
                escaped = false
            } else if c == '\\' {
                escaped = true
            } else if c == '"' {
                quoted = false
            };
            continue;
        }
        match c {
            '"' => quoted = true,
            '[' | '{' => depth += 1,
            ']' | '}' => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    anyhow!("error[parse:sparse-toon]: unbalanced recursive object")
                })?
            }
            ':' if depth == 0 => return Ok((&value[..index], &value[index + 1..])),
            _ => {}
        }
    }
    bail!("error[parse:sparse-toon]: invalid recursive object")
}
fn resolve_node(
    id: usize,
    nodes: &HashMap<usize, Node>,
    visiting: &mut HashSet<usize>,
) -> Result<Value> {
    if !visiting.insert(id) {
        bail!("error[parse:sparse-toon]: cyclic recursive reference");
    }
    let node = nodes
        .get(&id)
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: unknown recursive reference ^{id}"))?;
    let value = match node {
        Node::Object(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, cell)| Ok((key.clone(), resolve_cell(cell, nodes, visiting)?)))
                .collect::<Result<Map<_, _>>>()?,
        ),
        Node::List(cells) => Value::Array(
            cells
                .iter()
                .map(|cell| resolve_cell(cell, nodes, visiting))
                .collect::<Result<Vec<_>>>()?,
        ),
        Node::Table { columns, rows } => Value::Array(
            rows.iter()
                .map(|row| {
                    let mut object = Map::new();
                    for (path, cell) in columns.iter().zip(row) {
                        if !matches!(cell, Cell::Missing) {
                            set_pointer(&mut object, path, resolve_cell(cell, nodes, visiting)?)?;
                        }
                    }
                    Ok(Value::Object(object))
                })
                .collect::<Result<Vec<_>>>()?,
        ),
    };
    visiting.remove(&id);
    Ok(value)
}
fn resolve_cell(
    cell: &Cell,
    nodes: &HashMap<usize, Node>,
    visiting: &mut HashSet<usize>,
) -> Result<Value> {
    match cell {
        Cell::Value(value) => Ok(value.clone()),
        Cell::Ref(id) => resolve_node(*id, nodes, visiting),
        Cell::Missing => bail!("error[parse:sparse-toon]: missing value outside table"),
    }
}

fn decode_legacy_v1(body: &str) -> Result<Value> {
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
            if cell != "~" {
                set_pointer(
                    &mut row,
                    path,
                    serde_json::from_str(&cell)
                        .context("error[parse:sparse-toon]: invalid JSON cell")?,
                )?;
            }
        }
        rows.push(Value::Object(row));
    }
    Ok(Value::Array(rows))
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
        .parse()
        .context("error[parse:sparse-toon]: invalid row count")?;
    let columns = serde_json::from_str::<Vec<String>>(&format!("[{columns}]"))
        .context("error[parse:sparse-toon]: invalid column list")?;
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
                escaped = false
            } else if character == '\\' {
                escaped = true
            } else if character == '"' {
                quoted = false
            }
            continue;
        }
        match character {
            '"' => quoted = true,
            '[' | '{' => depth += 1,
            ']' | '}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| anyhow!("error[parse:sparse-toon]: unbalanced cell"))?
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
