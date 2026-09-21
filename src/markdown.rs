use anyhow::{anyhow, Result};
use jaq_json::Val;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde_json::{json, Value};

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

struct CodeState {
    lang: Option<String>,
    text: String,
}

struct ListState {
    ordered: bool,
    start: Option<u64>,
    depth: usize,
    items: Vec<String>,
    current_item: Option<String>,
}

struct TableState {
    in_head: bool,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    current_row: Vec<String>,
    current_cell: Option<String>,
}

struct LinkState {
    href: String,
    title: String,
    text: String,
}

struct Builder {
    title: Option<String>,
    frontmatter: Option<String>,
    sections: Vec<Section>,
    current: Section,
    paragraph: Option<String>,
    heading: Option<(u8, String)>,
    code: Option<CodeState>,
    list: Option<ListState>,
    quote: Option<String>,
    table: Option<TableState>,
    metadata: Option<String>,
    link: Option<LinkState>,
    links: Vec<Value>,
}

impl Builder {
    fn new() -> Self {
        Self {
            title: None,
            frontmatter: None,
            sections: Vec::new(),
            current: Section::default(),
            paragraph: None,
            heading: None,
            code: None,
            list: None,
            quote: None,
            table: None,
            metadata: None,
            link: None,
            links: Vec::new(),
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                if self.list.is_none()
                    && self.quote.is_none()
                    && self.table.is_none()
                    && self.metadata.is_none()
                {
                    self.paragraph = Some(String::new());
                }
            }
            Tag::Heading { level, .. } => {
                self.heading = Some((level as u8, String::new()));
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Indented => None,
                    CodeBlockKind::Fenced(info) => {
                        let lang = info.split_whitespace().next().unwrap_or_default().trim();
                        (!lang.is_empty()).then(|| lang.to_owned())
                    }
                };
                self.code = Some(CodeState {
                    lang,
                    text: String::new(),
                });
            }
            Tag::BlockQuote(_) => self.quote = Some(String::new()),
            Tag::List(start) => match self.list.as_mut() {
                Some(list) => list.depth += 1,
                None => {
                    self.list = Some(ListState {
                        ordered: start.is_some(),
                        start,
                        depth: 1,
                        items: Vec::new(),
                        current_item: None,
                    })
                }
            },
            Tag::Item => {
                if let Some(list) = self.list.as_mut() {
                    if list.current_item.is_none() {
                        list.current_item = Some(String::new());
                    }
                }
            }
            Tag::Table(_) => {
                self.table = Some(TableState {
                    in_head: false,
                    headers: Vec::new(),
                    rows: Vec::new(),
                    current_row: Vec::new(),
                    current_cell: None,
                });
            }
            Tag::TableHead => {
                if let Some(table) = self.table.as_mut() {
                    table.in_head = true;
                    table.current_row.clear();
                }
            }
            Tag::TableRow => {
                if let Some(table) = self.table.as_mut() {
                    table.current_row.clear();
                }
            }
            Tag::TableCell => {
                if let Some(table) = self.table.as_mut() {
                    table.current_cell = Some(String::new());
                }
            }
            Tag::Link {
                dest_url, title, ..
            } => {
                self.link = Some(LinkState {
                    href: dest_url.into_string(),
                    title: title.into_string(),
                    text: String::new(),
                });
            }
            Tag::MetadataBlock(_) => self.metadata = Some(String::new()),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                if let Some(text) = self.paragraph.take() {
                    self.push_text_block("paragraph", text);
                }
            }
            TagEnd::Heading(_) => {
                if let Some((level, text)) = self.heading.take() {
                    let text = clean(text);
                    if !self.current.is_empty() {
                        self.sections.push(std::mem::take(&mut self.current));
                    }
                    if level == 1 && self.title.is_none() {
                        self.title = Some(text.clone());
                    }
                    self.current.heading = Some(text);
                    self.current.level = level;
                }
            }
            TagEnd::CodeBlock => {
                if let Some(code) = self.code.take() {
                    self.current.blocks.push(json!({
                        "type": "code",
                        "lang": code.lang,
                        "text": code.text.trim_end_matches('\n'),
                    }));
                }
            }
            TagEnd::BlockQuote(_) => {
                if let Some(text) = self.quote.take() {
                    self.push_text_block("blockquote", text);
                }
            }
            TagEnd::Item => {
                if let Some(list) = self.list.as_mut() {
                    if let Some(item) = list.current_item.take() {
                        let item = clean(item);
                        if !item.is_empty() {
                            list.items.push(item);
                        }
                    }
                }
            }
            TagEnd::List(_) => {
                let nested = self.list.as_ref().is_some_and(|list| list.depth > 1);
                if nested {
                    if let Some(list) = self.list.as_mut() {
                        list.depth -= 1;
                    }
                } else if let Some(list) = self.list.take() {
                    self.current.blocks.push(json!({
                        "type": "list",
                        "ordered": list.ordered,
                        "start": list.start,
                        "items": list.items,
                    }));
                }
            }
            TagEnd::TableCell => {
                if let Some(table) = self.table.as_mut() {
                    if let Some(cell) = table.current_cell.take() {
                        table.current_row.push(clean(cell));
                    }
                }
            }
            TagEnd::TableHead => {
                if let Some(table) = self.table.as_mut() {
                    if !table.current_row.is_empty() {
                        table.headers = std::mem::take(&mut table.current_row);
                    }
                    table.in_head = false;
                }
            }
            TagEnd::TableRow => {
                if let Some(table) = self.table.as_mut() {
                    if table.in_head {
                        table.headers = std::mem::take(&mut table.current_row);
                    } else if !table.current_row.is_empty() {
                        table.rows.push(std::mem::take(&mut table.current_row));
                    }
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.current.blocks.push(json!({
                        "type": "table",
                        "headers": table.headers,
                        "rows": table.rows,
                    }));
                }
            }
            TagEnd::Link => {
                if let Some(link) = self.link.take() {
                    self.links.push(json!({
                        "text": clean(link.text),
                        "href": link.href,
                        "title": if link.title.is_empty() { Value::Null } else { Value::String(link.title) },
                    }));
                }
            }
            TagEnd::MetadataBlock(_) => {
                if let Some(metadata) = self.metadata.take() {
                    let metadata = metadata.trim().to_owned();
                    if !metadata.is_empty() {
                        self.frontmatter = Some(metadata);
                    }
                }
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if let Some(link) = self.link.as_mut() {
            link.text.push_str(text);
        }
        if let Some(metadata) = self.metadata.as_mut() {
            metadata.push_str(text);
        } else if let Some(table) = self.table.as_mut() {
            if let Some(cell) = table.current_cell.as_mut() {
                cell.push_str(text);
            }
        } else if let Some(code) = self.code.as_mut() {
            code.text.push_str(text);
        } else if let Some((_, heading)) = self.heading.as_mut() {
            heading.push_str(text);
        } else if let Some(list) = self.list.as_mut() {
            if let Some(item) = list.current_item.as_mut() {
                item.push_str(text);
            }
        } else if let Some(quote) = self.quote.as_mut() {
            quote.push_str(text);
        } else if let Some(paragraph) = self.paragraph.as_mut() {
            paragraph.push_str(text);
        }
    }

    fn separator(&mut self, separator: &str) {
        self.text(separator);
    }

    fn task_marker(&mut self, checked: bool) {
        if let Some(list) = self.list.as_mut() {
            if let Some(item) = list.current_item.as_mut() {
                item.push_str(if checked { "[x] " } else { "[ ] " });
            }
        }
    }

    fn html(&mut self, html: &str, inline: bool) {
        if inline {
            self.text(html);
        } else {
            let html = html.trim();
            if !html.is_empty() {
                self.current
                    .blocks
                    .push(json!({"type": "html", "text": html}));
            }
        }
    }

    fn rule(&mut self) {
        self.current.blocks.push(json!({"type": "rule"}));
    }

    fn push_text_block(&mut self, kind: &str, text: String) {
        let text = clean(text);
        if !text.is_empty() {
            self.current
                .blocks
                .push(json!({"type": kind, "text": text}));
        }
    }

    fn finish(mut self) -> Value {
        if !self.current.is_empty() {
            self.sections.push(self.current);
        }

        json!({
            "type": "markdown",
            "title": self.title,
            "frontmatter": self.frontmatter,
            "sections": self.sections.into_iter().map(Section::into_value).collect::<Vec<_>>(),
            "links": self.links,
        })
    }
}

pub fn parse(input: &str) -> Result<Val> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let mut builder = Builder::new();
    for event in Parser::new_ext(input, options) {
        match event {
            Event::Start(tag) => builder.start(tag),
            Event::End(tag) => builder.end(tag),
            Event::Text(text) | Event::Code(text) => builder.text(&text),
            Event::SoftBreak => builder.separator(" "),
            Event::HardBreak => builder.separator("\n"),
            Event::Html(html) => builder.html(&html, false),
            Event::InlineHtml(html) => builder.html(&html, true),
            Event::Rule => builder.rule(),
            Event::TaskListMarker(checked) => builder.task_marker(checked),
            Event::FootnoteReference(label) => builder.text(&format!("[^{label}]")),
            Event::InlineMath(math) => builder.text(&math),
            Event::DisplayMath(math) => builder.text(&math),
        }
    }

    serde_json::from_value(builder.finish())
        .map_err(|error| anyhow!("error[parse:markdown]: could not normalize document: {error}"))
}

fn clean(text: String) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
