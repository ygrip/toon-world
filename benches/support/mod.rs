use serde_json::{json, Map, Value};

pub const ROW_COUNTS: [usize; 3] = [10, 100, 1_000];
pub const SPARSITY_PERCENTS: [u8; 3] = [10, 30, 70];
pub const SHAPES: [Shape; 4] = [
    Shape::Shallow,
    Shape::Deep,
    Shape::Children,
    Shape::RootObject,
];

#[derive(Clone, Copy)]
pub enum Shape {
    Shallow,
    Deep,
    Children,
    /// A root object (not an array) holding a nested item array plus sibling
    /// scalar/object fields, matching an API response like
    /// `{"items": [...], "scope_counts": {...}}`.
    RootObject,
}

impl Shape {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Shallow => "flat",
            Self::Deep => "nested",
            Self::Children => "child-tables",
            Self::RootObject => "root-object",
        }
    }
}

pub struct Fixture {
    pub name: String,
    pub value: Value,
}

pub fn fixtures() -> Vec<Fixture> {
    let mut fixtures = Vec::new();
    for rows in ROW_COUNTS {
        for sparsity in SPARSITY_PERCENTS {
            for shape in SHAPES {
                fixtures.push(Fixture {
                    name: format!("{}-r{rows}-missing{sparsity}pct", shape.name()),
                    value: generate(rows, sparsity, shape),
                });
            }
        }
    }
    fixtures
}

fn generate(rows: usize, sparsity: u8, shape: Shape) -> Value {
    if let Shape::RootObject = shape {
        return root_object(rows, sparsity);
    }
    Value::Array(
        (0..rows)
            .map(|row| match shape {
                Shape::Shallow => shallow_row(row, sparsity),
                Shape::Deep => deep_row(row, sparsity),
                Shape::Children => children_row(row, sparsity),
                Shape::RootObject => unreachable!("handled above"),
            })
            .collect(),
    )
}

fn present(row: usize, field: usize, sparsity: u8) -> bool {
    (row.wrapping_mul(31).wrapping_add(field * 17) % 100) >= usize::from(sparsity)
}

fn shallow_row(row: usize, sparsity: u8) -> Value {
    let mut object = Map::from_iter([(String::from("id"), json!(row))]);
    for field in 0..12 {
        if present(row, field, sparsity) {
            object.insert(
                format!("field_{field}"),
                json!(format!("value-{row}-{field}")),
            );
        }
    }
    Value::Object(object)
}

fn deep_row(row: usize, sparsity: u8) -> Value {
    let mut profile = Map::new();
    let mut identity = Map::new();
    let mut location = Map::new();
    for field in 0..4 {
        if present(row, field, sparsity) {
            identity.insert(
                format!("name_{field}"),
                json!(format!("user-{row}-{field}")),
            );
        }
        if present(row, field + 4, sparsity) {
            location.insert(
                format!("place_{field}"),
                json!(format!("place-{row}-{field}")),
            );
        }
    }
    if !identity.is_empty() {
        profile.insert(String::from("identity"), Value::Object(identity));
    }
    if !location.is_empty() {
        profile.insert(String::from("location"), Value::Object(location));
    }
    let mut object = Map::from_iter([(String::from("id"), json!(row))]);
    if !profile.is_empty() {
        object.insert(String::from("profile"), Value::Object(profile));
    }
    Value::Object(object)
}

fn children_row(row: usize, sparsity: u8) -> Value {
    let children: Vec<Value> = (0..3)
        .map(|child| {
            let mut item = Map::from_iter([
                (String::from("id"), json!(child)),
                (String::from("kind"), json!("event")),
            ]);
            for field in 0..6 {
                if present(row + child, field, sparsity) {
                    item.insert(
                        format!("attribute_{field}"),
                        json!(format!("{row}-{child}-{field}")),
                    );
                }
            }
            Value::Object(item)
        })
        .collect();
    json!({"id": row, "events": children})
}

fn root_object(rows: usize, sparsity: u8) -> Value {
    let items: Vec<Value> = (0..rows).map(|row| delivery_item(row, sparsity)).collect();
    Value::Object(Map::from_iter([
        (String::from("items"), Value::Array(items)),
        (
            String::from("scope_counts"),
            json!({"active": rows.min(7), "completed": rows.saturating_sub(7)}),
        ),
    ]))
}

fn delivery_item(row: usize, sparsity: u8) -> Value {
    let mut object = Map::from_iter([
        (String::from("id"), json!(format!("delivery-{row}"))),
        (String::from("status"), json!("running")),
    ]);
    let mut usage = Map::new();
    if present(row, 0, sparsity) {
        usage.insert(String::from("tokens"), json!(row * 100));
    }
    if present(row, 1, sparsity) {
        usage.insert(String::from("cost"), json!(format!("{row}.00")));
    }
    let agents: Vec<Value> = (0..2)
        .filter(|agent| present(row, agent + 2, sparsity))
        .map(|agent| json!({"id": format!("agent-{agent}"), "tokens": row * 10 + agent}))
        .collect();
    if !agents.is_empty() {
        usage.insert(String::from("by_agent"), Value::Array(agents));
    }
    if !usage.is_empty() {
        object.insert(String::from("usage"), Value::Object(usage));
    }
    Value::Object(object)
}
