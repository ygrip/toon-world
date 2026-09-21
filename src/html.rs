use anyhow::{anyhow, Result};
use jaq_json::Val;
use scraper::{ElementRef, Html, Selector};
use serde_json::{json, Map, Value};

#[derive(Default)]
struct Section {
    heading: Option<String>,
    level: u8,
    blocks: Vec<Value>,
}

impl Section {
    fn is_empty(&self) -> bool {
        self.heading.is_none() && self.blocks.is_empty()
    }

    fn into_value(self) -> Value {
        json!({
            "heading": self.heading,
            "level": self.level,
            "blocks": self.blocks,
        })
    }
}

pub fn parse(input: &str, semantic: bool) -> Result<Val> {
    let document = Html::parse_document(input);
    let value = if semantic {
        semantic_document(&document)
    } else {
        structural_document(&document)
    };

    serde_json::from_value(value)
        .map_err(|error| anyhow!("error[parse:html]: could not normalize document: {error}"))
}

fn structural_document(document: &Html) -> Value {
    json!({
        "type": "html",
        "mode": "structural",
        "root": structural_element(document.root_element()),
    })
}

fn structural_element(element: ElementRef<'_>) -> Value {
    let attrs = element
        .value()
        .attrs()
        .map(|(name, value)| (name.to_owned(), Value::String(value.to_owned())))
        .collect::<Map<String, Value>>();

    let children = element
        .children()
        .filter_map(|child| {
            if let Some(element) = ElementRef::wrap(child) {
                return Some(structural_element(element));
            }
            if let Some(text) = child.value().as_text() {
                let text = text.to_string();
                if text.trim().is_empty() {
                    return None;
                }
                return Some(json!({"text": text}));
            }
            if let Some(comment) = child.value().as_comment() {
                return Some(json!({"comment": comment.to_string()}));
            }
            None
        })
        .collect::<Vec<_>>();

    json!({
        "tag": element.value().name(),
        "attrs": attrs,
        "children": children,
    })
}

fn semantic_document(document: &Html) -> Value {
    let title_selector = Selector::parse("title").expect("static title selector");
    let title = document
        .select(&title_selector)
        .next()
        .map(text)
        .filter(|text| !text.is_empty());

    json!({
        "type": "html",
        "mode": "semantic",
        "title": title,
        "metadata": semantic_metadata(document),
        "sections": semantic_sections(document),
        "links": semantic_links(document),
        "images": semantic_images(document),
        "forms": semantic_forms(document),
    })
}

fn semantic_sections(document: &Html) -> Vec<Value> {
    let selector = Selector::parse("h1,h2,h3,h4,h5,h6,p,pre,ul,ol,table,blockquote")
        .expect("static HTML block selector");
    let mut sections = Vec::new();
    let mut current = Section::default();

    for element in document.select(&selector) {
        let tag = element.value().name();
        if is_nested_block(element) {
            continue;
        }

        if let Some(level) = heading_level(tag) {
            if !current.is_empty() {
                sections.push(std::mem::take(&mut current).into_value());
            }
            current.heading = Some(text(element));
            current.level = level;
            continue;
        }

        let block = match tag {
            "p" => non_empty_text_block("paragraph", text(element)),
            "blockquote" => non_empty_text_block("blockquote", text(element)),
            "pre" => Some(code_block(element)),
            "ul" | "ol" => Some(list_block(element, tag == "ol")),
            "table" => Some(table_block(element)),
            _ => None,
        };
        if let Some(block) = block {
            current.blocks.push(block);
        }
    }

    if !current.is_empty() {
        sections.push(current.into_value());
    }
    sections
}

fn is_nested_block(element: ElementRef<'_>) -> bool {
    let tag = element.value().name();
    let containers: &[&str] = match tag {
        "p" => &["li", "blockquote", "table"],
        "ul" | "ol" => &["li"],
        "pre" => &["blockquote"],
        _ => &[],
    };
    if containers.is_empty() {
        return false;
    }

    let mut parent = element.parent();
    while let Some(node) = parent {
        if let Some(parent_element) = ElementRef::wrap(node) {
            if containers.contains(&parent_element.value().name()) {
                return true;
            }
        }
        parent = node.parent();
    }
    false
}

fn code_block(element: ElementRef<'_>) -> Value {
    let code_selector = Selector::parse("code").expect("static code selector");
    let code = element.select(&code_selector).next();
    let lang = code.and_then(|code| {
        code.value()
            .classes()
            .find_map(|class| class.strip_prefix("language-").map(str::to_owned))
    });
    let content = code.map(text).unwrap_or_else(|| text(element));

    json!({
        "type": "code",
        "lang": lang,
        "text": content,
    })
}

fn list_block(element: ElementRef<'_>, ordered: bool) -> Value {
    let items = element
        .child_elements()
        .filter(|child| child.value().name() == "li")
        .map(text)
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    let start = if ordered {
        element
            .value()
            .attr("start")
            .and_then(|value| value.parse::<u64>().ok())
    } else {
        None
    };

    json!({
        "type": "list",
        "ordered": ordered,
        "start": start,
        "items": items,
    })
}

fn table_block(element: ElementRef<'_>) -> Value {
    let row_selector = Selector::parse("tr").expect("static row selector");
    let cell_selector = Selector::parse("th,td").expect("static cell selector");
    let th_selector = Selector::parse("th").expect("static th selector");
    let mut headers = Vec::new();
    let mut rows = Vec::new();

    for row in element.select(&row_selector) {
        let cells = row.select(&cell_selector).map(text).collect::<Vec<_>>();
        if headers.is_empty() && row.select(&th_selector).next().is_some() {
            headers = cells;
        } else if !cells.is_empty() {
            rows.push(cells);
        }
    }

    json!({
        "type": "table",
        "headers": headers,
        "rows": rows,
    })
}

fn semantic_links(document: &Html) -> Vec<Value> {
    let selector = Selector::parse("a[href]").expect("static link selector");
    document
        .select(&selector)
        .map(|element| {
            json!({
                "text": text(element),
                "href": element.value().attr("href").unwrap_or_default(),
                "title": optional_attr(element, "title"),
            })
        })
        .collect()
}

fn semantic_images(document: &Html) -> Vec<Value> {
    let selector = Selector::parse("img[src]").expect("static image selector");
    document
        .select(&selector)
        .map(|element| {
            json!({
                "src": element.value().attr("src").unwrap_or_default(),
                "alt": element.value().attr("alt").unwrap_or_default(),
                "title": optional_attr(element, "title"),
            })
        })
        .collect()
}

fn semantic_metadata(document: &Html) -> Vec<Value> {
    let selector = Selector::parse("meta[name][content],meta[property][content]")
        .expect("static metadata selector");
    document
        .select(&selector)
        .map(|element| {
            json!({
                "key": element
                    .value()
                    .attr("name")
                    .or_else(|| element.value().attr("property"))
                    .unwrap_or_default(),
                "content": element.value().attr("content").unwrap_or_default(),
            })
        })
        .collect()
}

fn semantic_forms(document: &Html) -> Vec<Value> {
    let form_selector = Selector::parse("form").expect("static form selector");
    let field_selector =
        Selector::parse("input,select,textarea,button").expect("static field selector");
    let label_selector = Selector::parse("label").expect("static label selector");

    document
        .select(&form_selector)
        .map(|form| {
            let fields = form
                .select(&field_selector)
                .map(|field| {
                    json!({
                        "tag": field.value().name(),
                        "name": optional_attr(field, "name"),
                        "type": optional_attr(field, "type"),
                        "value": optional_attr(field, "value"),
                        "placeholder": optional_attr(field, "placeholder"),
                    })
                })
                .collect::<Vec<_>>();
            let labels = form
                .select(&label_selector)
                .map(|label| {
                    json!({
                        "for": optional_attr(label, "for"),
                        "text": text(label),
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "action": optional_attr(form, "action"),
                "method": form.value().attr("method").unwrap_or("get").to_ascii_lowercase(),
                "fields": fields,
                "labels": labels,
            })
        })
        .collect()
}

fn optional_attr(element: ElementRef<'_>, name: &str) -> Value {
    element
        .value()
        .attr(name)
        .map(|value| Value::String(value.to_owned()))
        .unwrap_or(Value::Null)
}

fn heading_level(tag: &str) -> Option<u8> {
    match tag {
        "h1" => Some(1),
        "h2" => Some(2),
        "h3" => Some(3),
        "h4" => Some(4),
        "h5" => Some(5),
        "h6" => Some(6),
        _ => None,
    }
}

fn text(element: ElementRef<'_>) -> String {
    clean(element.text().collect::<Vec<_>>().join(" "))
}

fn clean(text: String) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn non_empty_text_block(kind: &str, text: String) -> Option<Value> {
    (!text.is_empty()).then(|| json!({"type": kind, "text": text}))
}
