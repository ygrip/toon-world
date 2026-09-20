use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Map, Value};

const HEADER: &str = "@toon-world/sparse-v1\n";
const MAX_DEPTH: usize = 64;

#[derive(Clone, Debug)]
struct Field {
    key: String,
    kind: FieldKind,
}

#[derive(Clone, Debug)]
enum FieldKind {
    Leaf,
    Object(Schema),
    Table(Schema),
}

#[derive(Clone, Debug, Default)]
struct Schema {
    fields: Vec<Field>,
    indices: HashMap<String, usize>,
}

impl Schema {
    fn from_fields(fields: Vec<Field>) -> Self {
        let indices = fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.key.clone(), index))
            .collect();
        Self { fields, indices }
    }
}

#[derive(Debug)]
struct Line {
    indent: usize,
    text: String,
}

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
/// Encodes root arrays of objects as headerless sparse TOON. Other shapes use standard TOON.
pub fn encode(value: &Value) -> Result<Option<String>> {
    let Value::Array(values) = value else {
        return Ok(None);
    };
    if values.is_empty() || !values.iter().all(Value::is_object) {
        return Ok(None);
    }
    let mut output = String::new();
    if values
        .iter()
        .all(|value| value.as_object().is_some_and(Map::is_empty))
    {
        render_empty_object_table(None, values.len(), 0, &mut output)?;
    } else {
        let Some(schema) = schema_for_rows(values)? else {
            return Ok(None);
        };
        render_table(None, values, &schema, 0, &mut output)?;
    }
    Ok(Some(output))
}

fn schema_for_rows(values: &[Value]) -> Result<Option<Schema>> {
    let mut schema = Schema::default();
    for value in values {
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("error[encode:sparse-toon]: table row must be an object"))?;
        if has_unsupported_keys(value) {
            return Ok(None);
        }
        if !merge_schema(&mut schema, object, 0)? {
            return Ok(None);
        }
    }
    if values.iter().any(|value| {
        value
            .as_object()
            .is_some_and(|object| !row_matches_schema(&schema, object))
    }) {
        return Ok(None);
    }
    Ok((!schema.fields.is_empty()).then_some(schema))
}

fn has_unsupported_keys(value: &Value) -> bool {
    match value {
        Value::Array(values) => {
            let homogeneous = values.iter().all(Value::is_object)
                || values
                    .iter()
                    .all(|value| !value.is_array() && !value.is_object());
            !homogeneous || values.iter().any(has_unsupported_keys)
        }
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| key.is_empty() || has_unsupported_keys(value)),
        _ => false,
    }
}

fn row_matches_schema(schema: &Schema, object: &Map<String, Value>) -> bool {
    schema
        .fields
        .iter()
        .all(|field| match (&field.kind, object.get(&field.key)) {
            (_, None) => true,
            (FieldKind::Leaf, Some(value)) => !value.is_array() && !value.is_object(),
            (FieldKind::Object(nested), Some(Value::Object(child))) => {
                row_matches_schema(nested, child)
            }
            (FieldKind::Table(nested), Some(Value::Array(items))) => items.iter().all(|item| {
                item.as_object()
                    .is_some_and(|item| row_matches_schema(nested, item))
            }),
            _ => false,
        })
}

fn merge_schema(schema: &mut Schema, object: &Map<String, Value>, depth: usize) -> Result<bool> {
    if depth >= MAX_DEPTH {
        return Ok(false);
    }
    for (key, value) in object {
        let observed = match value {
            Value::Array(items) if !items.is_empty() && items.iter().all(Value::is_object) => {
                let mut nested = Schema::default();
                for item in items {
                    let item = item
                        .as_object()
                        .expect("checked all items are objects above");
                    if !merge_schema(&mut nested, item, depth + 1)? {
                        return Ok(false);
                    }
                }
                if nested.fields.is_empty() {
                    continue;
                }
                FieldKind::Table(nested)
            }
            Value::Array(_) => continue,
            Value::Object(map) if map.is_empty() => continue,
            Value::Object(map) => {
                let mut nested = Schema::default();
                if !merge_schema(&mut nested, map, depth + 1)? {
                    return Ok(false);
                }
                if nested.fields.is_empty() {
                    continue;
                }
                FieldKind::Object(nested)
            }
            _ => FieldKind::Leaf,
        };
        match schema.indices.get(key).copied() {
            Some(index) => match (&mut schema.fields[index].kind, observed) {
                (FieldKind::Object(existing), FieldKind::Object(observed)) => {
                    if !merge_fields(existing, observed) {
                        return Ok(false);
                    }
                }
                (FieldKind::Table(existing), FieldKind::Table(observed)) => {
                    if !merge_fields(existing, observed) {
                        return Ok(false);
                    }
                }
                (FieldKind::Leaf, FieldKind::Leaf) => {}
                _ => return Ok(false),
            },
            None => {
                let index = schema.fields.len();
                schema.indices.insert(key.clone(), index);
                schema.fields.push(Field {
                    key: key.clone(),
                    kind: observed,
                });
            }
        }
    }
    Ok(true)
}

fn merge_fields(existing: &mut Schema, observed: Schema) -> bool {
    for field in observed.fields {
        match existing.indices.get(&field.key).copied() {
            Some(index) => match (&mut existing.fields[index].kind, field.kind) {
                (FieldKind::Object(current), FieldKind::Object(nested)) => {
                    if !merge_fields(current, nested) {
                        return false;
                    }
                }
                (FieldKind::Table(current), FieldKind::Table(nested)) => {
                    if !merge_fields(current, nested) {
                        return false;
                    }
                }
                (FieldKind::Leaf, FieldKind::Leaf) => {}
                _ => return false,
            },
            None => {
                let index = existing.fields.len();
                existing.indices.insert(field.key.clone(), index);
                existing.fields.push(field);
            }
        }
    }
    true
}

fn render_table(
    key: Option<&str>,
    values: &[Value],
    schema: &Schema,
    indent: usize,
    output: &mut String,
) -> Result<()> {
    output.push_str(&" ".repeat(indent));
    if let Some(key) = key {
        output.push_str(&render_key(key)?);
    }
    output.push_str(&format!("[{}]{{", values.len()));
    render_fields(schema, output)?;
    output.push_str("}:\n");
    render_table_rows(schema, values, indent + 2, output)
}

fn render_table_rows(
    schema: &Schema,
    values: &[Value],
    indent: usize,
    output: &mut String,
) -> Result<()> {
    for value in values {
        let object = value
            .as_object()
            .ok_or_else(|| anyhow!("error[encode:sparse-toon]: table row must be an object"))?;
        let mut cells = Vec::new();
        collect_cells(&schema.fields, object, &mut cells)?;
        output.push_str(&" ".repeat(indent));
        if cells.is_empty() {
            // A row line must not be blank: parse_lines() treats whitespace-only
            // lines as filler and drops them, which would lose this row entirely.
            output.push('~');
        } else {
            output.push_str(&cells.join(","));
        }
        output.push('\n');
        render_attachments(schema, object, indent + 2, output)?;
    }
    Ok(())
}

fn render_empty_object_table(
    key: Option<&str>,
    count: usize,
    indent: usize,
    output: &mut String,
) -> Result<()> {
    output.push_str(&" ".repeat(indent));
    if let Some(key) = key {
        output.push_str(&render_key(key)?);
    }
    output.push_str(&format!("[{count}]{{}}:\n"));
    Ok(())
}

fn render_fields(schema: &Schema, output: &mut String) -> Result<()> {
    for (index, field) in schema.fields.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&render_key(&field.key)?);
        match &field.kind {
            FieldKind::Object(nested) => {
                output.push('{');
                render_fields(nested, output)?;
                output.push('}');
            }
            FieldKind::Table(nested) => {
                output.push_str("[]{");
                render_fields(nested, output)?;
                output.push('}');
            }
            FieldKind::Leaf => {}
        }
    }
    Ok(())
}

fn collect_cells(
    fields: &[Field],
    object: &Map<String, Value>,
    cells: &mut Vec<String>,
) -> Result<()> {
    for field in fields {
        match &field.kind {
            FieldKind::Leaf => match object.get(&field.key) {
                Some(value) if !value.is_array() && !value.is_object() => {
                    cells.push(render_primitive(value)?)
                }
                None => cells.push("~".to_owned()),
                Some(_) => bail!("error[encode:sparse-toon]: conflicting table row shape"),
            },
            FieldKind::Object(nested) => match object.get(&field.key) {
                Some(Value::Object(child)) => collect_cells(&nested.fields, child, cells)?,
                None => cells.extend((0..leaf_count(nested)).map(|_| "~".to_owned())),
                Some(_) => bail!("error[encode:sparse-toon]: conflicting table row shape"),
            },
            FieldKind::Table(_) => {}
        }
    }
    Ok(())
}

fn render_primitive(value: &Value) -> Result<String> {
    if let Value::String(string) = value {
        return render_string(string);
    }
    let rendered = toon_format::encode_default(value)
        .map_err(|error| anyhow!("error[encode:sparse-toon]: {error}"))?;
    Ok(rendered)
}

fn render_string(value: &str) -> Result<String> {
    let mut characters = value.chars();
    let bare = matches!(characters.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/' | '@')
        })
        && !matches!(value, "null" | "true" | "false" | "~");
    if bare {
        Ok(value.to_owned())
    } else {
        serde_json::to_string(value).context("error[encode:sparse-toon]: invalid string value")
    }
}

fn render_key(key: &str) -> Result<String> {
    let mut chars = key.chars();
    if matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Ok(key.to_owned());
    }
    serde_json::to_string(key).context("error[encode:sparse-toon]: invalid field name")
}

fn render_attachments(
    schema: &Schema,
    object: &Map<String, Value>,
    indent: usize,
    output: &mut String,
) -> Result<()> {
    for (key, value) in object {
        match schema.indices.get(key).map(|&index| &schema.fields[index].kind) {
            Some(FieldKind::Leaf) => {}
            Some(FieldKind::Table(nested)) => {
                let items = value.as_array().ok_or_else(|| {
                    anyhow!("error[encode:sparse-toon]: conflicting table row shape")
                })?;
                output.push_str(&" ".repeat(indent));
                output.push_str(&render_key(key)?);
                output.push_str(&format!("[{}]:\n", items.len()));
                render_table_rows(nested, items, indent + 2, output)?;
            }
            Some(FieldKind::Object(nested)) => {
                let child = value.as_object().ok_or_else(|| {
                    anyhow!("error[encode:sparse-toon]: conflicting table row shape")
                })?;
                if child.is_empty() {
                    output.push_str(&format!("{}{}:\n", " ".repeat(indent), render_key(key)?));
                } else {
                    let mut nested_output = String::new();
                    render_attachments(nested, child, indent + 2, &mut nested_output)?;
                    if !nested_output.is_empty() {
                        output.push_str(&format!("{}{}:\n", " ".repeat(indent), render_key(key)?));
                        output.push_str(&nested_output);
                    }
                }
            }
            None => match value {
                Value::Array(values) => render_array_attachment(key, values, indent, output)?,
                Value::Object(child) if child.is_empty() => {
                    output.push_str(&format!("{}{}:\n", " ".repeat(indent), render_key(key)?));
                }
                Value::Object(child) if has_attachments(child) => {
                    output.push_str(&format!("{}{}:\n", " ".repeat(indent), render_key(key)?));
                    render_attachments(&Schema::default(), child, indent + 2, output)?;
                }
                _ => {}
            },
        }
    }
    Ok(())
}

fn has_attachments(object: &Map<String, Value>) -> bool {
    object.values().any(|value| match value {
        Value::Array(_) => true,
        Value::Object(child) => child.is_empty() || has_attachments(child),
        _ => false,
    })
}

fn render_array_attachment(
    key: &str,
    values: &[Value],
    indent: usize,
    output: &mut String,
) -> Result<()> {
    if values.is_empty() {
        output.push_str(&format!("{}{}[0]:\n", " ".repeat(indent), render_key(key)?));
        return Ok(());
    }
    if values
        .iter()
        .all(|value| value.as_object().is_some_and(Map::is_empty))
    {
        return render_empty_object_table(Some(key), values.len(), indent, output);
    }
    if values.iter().all(Value::is_object) {
        if let Some(schema) = schema_for_rows(values)? {
            return render_table(Some(key), values, &schema, indent, output);
        }
    }
    let wrapped = Value::Object(Map::from_iter([(
        key.to_owned(),
        Value::Array(values.to_vec()),
    )]));
    let rendered = toon_format::encode_default(&wrapped)
        .map_err(|error| anyhow!("error[encode:sparse-toon]: {error}"))?;
    for line in rendered.lines() {
        output.push_str(&" ".repeat(indent));
        output.push_str(line);
        output.push('\n');
    }
    Ok(())
}

fn leaf_count(schema: &Schema) -> usize {
    schema
        .fields
        .iter()
        .map(|field| match &field.kind {
            FieldKind::Leaf => 1,
            FieldKind::Object(nested) => leaf_count(nested),
            FieldKind::Table(_) => 0,
        })
        .sum()
}

/// Decodes headerless sparse TOON and legacy sparse-v1. `None` means standard TOON.
pub fn decode(input: &str) -> Result<Option<Value>> {
    if let Some(body) = input.strip_prefix(HEADER) {
        return if body.starts_with("root=^") {
            decode_recursive_v1(body).map(Some)
        } else {
            decode_legacy_v1(body).map(Some)
        };
    }
    let Some(first) = input.lines().find(|line| !line.trim().is_empty()) else {
        return Ok(None);
    };
    let indent = first
        .chars()
        .take_while(|character| *character == ' ')
        .count();
    let Some(header) = parse_header(&first[indent..])? else {
        return Ok(None);
    };
    if indent != 0 || header.key.is_some() || header.fields.is_none() || !header.inline.is_empty() {
        return Ok(None);
    }
    let lines = parse_lines(input)?;
    let mut index = 0;
    let value = parse_table(&lines, &mut index, 0, false)?;
    if index != lines.len() {
        bail!("error[parse:sparse-toon]: trailing content after root table");
    }
    Ok(Some(value))
}

#[derive(Debug)]
struct HeaderLine {
    key: Option<String>,
    count: usize,
    fields: Option<Schema>,
    inline: String,
}

fn parse_lines(input: &str) -> Result<Vec<Line>> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let indent = line
                .chars()
                .take_while(|character| *character == ' ')
                .count();
            if line[..indent].contains('\t') || line[indent..].starts_with('\t') {
                bail!("error[parse:sparse-toon]: tabs are not valid indentation");
            }
            if indent > MAX_DEPTH * 2 {
                bail!("error[parse:sparse-toon]: maximum nesting depth exceeded");
            }
            Ok(Line {
                indent,
                text: line[indent..].to_owned(),
            })
        })
        .collect()
}

fn parse_header(input: &str) -> Result<Option<HeaderLine>> {
    let Some(bracket) = find_unquoted(input, '[') else {
        return Ok(None);
    };
    let Some(close) = input[bracket + 1..].find(']') else {
        bail!("error[parse:sparse-toon]: invalid array header");
    };
    let close = bracket + 1 + close;
    let key = if bracket == 0 {
        None
    } else {
        Some(parse_key(&input[..bracket])?)
    };
    let count = input[bracket + 1..close]
        .parse()
        .context("error[parse:sparse-toon]: invalid array length")?;
    let mut rest = &input[close + 1..];
    let fields = if rest.starts_with('{') {
        let end = matching_brace(rest)?;
        let fields = if end == 1 {
            Schema::default()
        } else {
            parse_fields(&rest[1..end], 0)?
        };
        rest = &rest[end + 1..];
        Some(fields)
    } else {
        None
    };
    let Some(inline) = rest.strip_prefix(':') else {
        bail!("error[parse:sparse-toon]: expected ':' after array header");
    };
    Ok(Some(HeaderLine {
        key,
        count,
        fields,
        inline: inline.trim_start().to_owned(),
    }))
}

fn parse_fields(input: &str, depth: usize) -> Result<Schema> {
    if input.is_empty() {
        bail!("error[parse:sparse-toon]: field list must not be empty");
    }
    if depth >= MAX_DEPTH {
        bail!("error[parse:sparse-toon]: maximum field depth exceeded");
    }
    let fields = split_cells(input)?
        .into_iter()
        .map(|entry| {
            if let Some(open) = find_unquoted(&entry, '{') {
                if matching_brace(&entry[open..])? != entry.len() - open - 1 {
                    bail!("error[parse:sparse-toon]: invalid nested field group");
                }
                let key_part = &entry[..open];
                let nested = parse_fields(&entry[open + 1..entry.len() - 1], depth + 1)?;
                if let Some(base) = key_part.strip_suffix("[]") {
                    Ok(Field {
                        key: parse_key(base)?,
                        kind: FieldKind::Table(nested),
                    })
                } else {
                    Ok(Field {
                        key: parse_key(key_part)?,
                        kind: FieldKind::Object(nested),
                    })
                }
            } else {
                Ok(Field {
                    key: parse_key(&entry)?,
                    kind: FieldKind::Leaf,
                })
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let mut seen = HashSet::new();
    if fields.iter().any(|field| !seen.insert(&field.key)) {
        bail!("error[parse:sparse-toon]: duplicate field name");
    }
    Ok(Schema::from_fields(fields))
}

fn matching_brace(input: &str) -> Result<usize> {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
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
            '{' => depth += 1,
            '}' => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| anyhow!("error[parse:sparse-toon]: unbalanced field group"))?;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => {}
        }
    }
    bail!("error[parse:sparse-toon]: unbalanced field group")
}

fn find_unquoted(input: &str, needle: char) -> Option<usize> {
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
        } else if character == '"' {
            quoted = true;
        } else if character == needle {
            return Some(index);
        }
    }
    None
}

fn parse_key(input: &str) -> Result<String> {
    if input.starts_with('"') {
        return serde_json::from_str(input)
            .context("error[parse:sparse-toon]: invalid quoted field name");
    }
    if input.is_empty() {
        bail!("error[parse:sparse-toon]: field name must not be empty");
    }
    Ok(input.to_owned())
}

fn parse_table(
    lines: &[Line],
    index: &mut usize,
    indent: usize,
    require_key: bool,
) -> Result<Value> {
    let line = lines
        .get(*index)
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: missing table header"))?;
    if line.indent != indent {
        bail!("error[parse:sparse-toon]: invalid table indentation");
    }
    let header = parse_header(&line.text)?
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: invalid table header"))?;
    if require_key != header.key.is_some() || !header.inline.is_empty() {
        bail!("error[parse:sparse-toon]: invalid sparse table header");
    }
    let fields = header
        .fields
        .ok_or_else(|| anyhow!("error[parse:sparse-toon]: sparse table requires fields"))?;
    *index += 1;
    let rows = parse_table_body(lines, index, indent + 2, header.count, &fields)?;
    Ok(Value::Array(rows))
}

fn parse_table_body(
    lines: &[Line],
    index: &mut usize,
    row_indent: usize,
    count: usize,
    fields: &Schema,
) -> Result<Vec<Value>> {
    if fields.fields.is_empty() {
        return Ok((0..count).map(|_| Value::Object(Map::new())).collect());
    }
    let mut rows = Vec::with_capacity(count.min(lines.len()));
    for _ in 0..count {
        let row_line = lines
            .get(*index)
            .ok_or_else(|| anyhow!("error[parse:sparse-toon]: missing table row"))?;
        if row_line.indent != row_indent {
            bail!("error[parse:sparse-toon]: invalid table row indentation");
        }
        let expected = leaf_count(fields);
        let cells = if expected == 0 {
            if row_line.text != "~" {
                bail!("error[parse:sparse-toon]: expected empty row marker '~'");
            }
            Vec::new()
        } else {
            let cells = split_cells(&row_line.text)?;
            if cells.len() != expected {
                bail!(
                    "error[parse:sparse-toon]: expected {} cells, found {}",
                    expected,
                    cells.len()
                );
            }
            cells
        };
        let mut cell_index = 0;
        let mut object = decode_row(fields, &cells, &mut cell_index)?;
        *index += 1;
        parse_attachments(lines, index, row_indent + 2, fields, &mut object)?;
        rows.push(Value::Object(object));
    }
    Ok(rows)
}

fn decode_row(fields: &Schema, cells: &[String], index: &mut usize) -> Result<Map<String, Value>> {
    let mut object = Map::new();
    for field in &fields.fields {
        match &field.kind {
            FieldKind::Leaf => {
                let cell = cells
                    .get(*index)
                    .ok_or_else(|| anyhow!("error[parse:sparse-toon]: missing row cell"))?;
                *index += 1;
                if cell != "~" {
                    let value = toon_format::decode_default(cell)
                        .map_err(|error| anyhow!("error[parse:sparse-toon]: {error}"))?;
                    object.insert(field.key.clone(), value);
                }
            }
            FieldKind::Object(nested) => {
                let child = decode_row(nested, cells, index)?;
                if !child.is_empty() {
                    object.insert(field.key.clone(), Value::Object(child));
                }
            }
            FieldKind::Table(_) => {}
        }
    }
    Ok(object)
}

fn parse_attachments(
    lines: &[Line],
    index: &mut usize,
    indent: usize,
    schema: &Schema,
    object: &mut Map<String, Value>,
) -> Result<()> {
    while let Some(line) = lines.get(*index) {
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            bail!("error[parse:sparse-toon]: unexpected attachment indentation");
        }
        if let Some(header) = parse_header(&line.text)? {
            if header.fields.is_some() && header.inline.is_empty() {
                let key = header.key.clone().ok_or_else(|| {
                    anyhow!("error[parse:sparse-toon]: child table requires a key")
                })?;
                let value = parse_table(lines, index, indent, true)?;
                merge_value(object, key, value)?;
                continue;
            }
            if header.fields.is_none() && header.count == 0 && header.inline.is_empty() {
                let key = header.key.ok_or_else(|| {
                    anyhow!("error[parse:sparse-toon]: child array requires a key")
                })?;
                *index += 1;
                merge_value(object, key, Value::Array(Vec::new()))?;
                continue;
            }
            if header.fields.is_none() && header.inline.is_empty() {
                if let Some(FieldKind::Table(nested)) = header
                    .key
                    .as_ref()
                    .and_then(|key| schema.indices.get(key))
                    .map(|&position| &schema.fields[position].kind)
                {
                    let key = header.key.clone().expect("checked above");
                    *index += 1;
                    let rows = parse_table_body(lines, index, indent + 2, header.count, nested)?;
                    merge_value(object, key, Value::Array(rows))?;
                    continue;
                }
            }
            parse_standard_attachment(lines, index, indent, object)?;
            continue;
        }
        if let Some((key, rest)) = split_key_line(&line.text)? {
            if rest.is_empty() {
                *index += 1;
                let mut child = match object.remove(&key) {
                    Some(Value::Object(child)) => child,
                    Some(_) => bail!("error[parse:sparse-toon]: conflicting attachment value"),
                    None => Map::new(),
                };
                let empty = Schema::default();
                let nested = match schema.indices.get(&key) {
                    Some(&position) => match &schema.fields[position].kind {
                        FieldKind::Object(nested) => nested,
                        _ => &empty,
                    },
                    None => &empty,
                };
                parse_attachments(lines, index, indent + 2, nested, &mut child)?;
                object.insert(key, Value::Object(child));
                continue;
            }
        }
        parse_standard_attachment(lines, index, indent, object)?;
    }
    Ok(())
}

fn split_key_line(input: &str) -> Result<Option<(String, &str)>> {
    let Some(colon) = find_unquoted(input, ':') else {
        return Ok(None);
    };
    Ok(Some((
        parse_key(&input[..colon])?,
        input[colon + 1..].trim_start(),
    )))
}

fn parse_standard_attachment(
    lines: &[Line],
    index: &mut usize,
    indent: usize,
    object: &mut Map<String, Value>,
) -> Result<()> {
    let start = *index;
    *index += 1;
    while let Some(line) = lines.get(*index) {
        if line.indent <= indent {
            break;
        }
        *index += 1;
    }
    let mut input = String::new();
    for line in &lines[start..*index] {
        if line.indent < indent {
            bail!("error[parse:sparse-toon]: invalid attachment indentation");
        }
        input.push_str(&" ".repeat(line.indent - indent));
        input.push_str(&line.text);
        input.push('\n');
    }
    let value: Value = toon_format::decode_default(&input)
        .map_err(|error| anyhow!("error[parse:sparse-toon]: {error}"))?;
    let Value::Object(decoded) = value else {
        bail!("error[parse:sparse-toon]: attachment must decode to an object");
    };
    for (key, value) in decoded {
        merge_value(object, key, value)?;
    }
    Ok(())
}

fn merge_value(object: &mut Map<String, Value>, key: String, value: Value) -> Result<()> {
    match (object.get_mut(&key), value) {
        (None, value) => {
            object.insert(key, value);
        }
        (Some(Value::Object(existing)), Value::Object(incoming)) => {
            for (nested_key, nested_value) in incoming {
                merge_value(existing, nested_key, nested_value)?;
            }
        }
        _ => bail!("error[parse:sparse-toon]: duplicate or conflicting attachment"),
    }
    Ok(())
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
            let mut rows = Vec::with_capacity(count.min(rest.lines().count()));
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
    let columns = split_cells(columns)?
        .into_iter()
        .map(|column| parse_column(&column))
        .collect::<Result<Vec<_>>>()?;
    if columns.is_empty() {
        bail!("error[parse:sparse-toon]: columns must be JSON Pointer paths");
    }
    Ok((count, columns))
}

fn parse_column(column: &str) -> Result<String> {
    if column.starts_with('"') {
        let pointer: String = serde_json::from_str(column)
            .context("error[parse:sparse-toon]: invalid quoted column")?;
        if pointer.starts_with('/') {
            return Ok(pointer);
        }
        bail!("error[parse:sparse-toon]: quoted column must be a JSON Pointer path");
    }
    let segments = column.split('.').collect::<Vec<_>>();
    if segments.is_empty()
        || !segments
            .iter()
            .all(|segment| is_bare_column_segment(segment))
    {
        bail!("error[parse:sparse-toon]: invalid bare column");
    }
    Ok(format!(
        "/{}",
        segments
            .into_iter()
            .map(escape_pointer)
            .collect::<Vec<_>>()
            .join("/")
    ))
}

fn is_bare_column_segment(segment: &str) -> bool {
    let mut characters = segment.chars();
    matches!(characters.next(), Some(character) if character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
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
