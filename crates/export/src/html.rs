// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Debug, Clone, PartialEq)]
pub struct MathRun {
    pub latex: String,
    pub ascii: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub code: bool,
    pub color: Option<String>,
    pub font_size: Option<f64>,
    pub font_family: Option<String>,
    pub link: Option<String>,
    pub math: Option<MathRun>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub rows: Vec<TableRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableCell {
    pub runs: Vec<Run>,
    pub header: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(Vec<Run>),
    Heading(u8, Vec<Run>),
    ListItem(Vec<Run>),
    Table(Table),
}

#[derive(Debug, Clone, Default)]
struct Style {
    bold: bool,
    italic: bool,
    underline: bool,
    code: bool,
    color: Option<String>,
    font_size: Option<f64>,
    font_family: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockKind {
    Paragraph,
    Heading(u8),
    ListItem,
}

/// A deliberately small HTML fragment parser for TipTap output.
///
/// It preserves paragraphs, headings, list items, bold, italic, underline,
/// inline code, links, text color, font size and tables. Unknown tags are
/// ignored. Embedded images are not handled here.
pub fn plain_text(html: &str) -> String {
    parse_html(html)
        .into_iter()
        .map(|block| match block {
            Block::Paragraph(runs) | Block::Heading(_, runs) | Block::ListItem(runs) => {
                runs.into_iter().map(|run| run.text).collect::<String>()
            }
            Block::Table(table) => table
                .rows
                .into_iter()
                .flat_map(|row| row.cells)
                .flat_map(|cell| cell.runs)
                .map(|run| run.text)
                .collect::<String>(),
        })
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_html(html: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut rest = html;

    while let Some(start) = find_open_tag(rest, "table") {
        blocks.extend(parse_plain(&rest[..start]));

        let Some(end) = find_matching_close(rest, start, "table") else {
            blocks.extend(parse_plain(&rest[start..]));
            return blocks;
        };

        if let Some(table) = parse_table(&rest[start..end]) {
            blocks.push(Block::Table(table));
        }
        rest = &rest[end..];
    }

    blocks.extend(parse_plain(rest));
    blocks
}

fn parse_plain(html: &str) -> Vec<Block> {
    let mut parser = Parser {
        blocks: Vec::new(),
        current: Vec::new(),
        kind: BlockKind::Paragraph,
        style: Style::default(),
        style_stack: Vec::new(),
        link: None,
        link_stack: Vec::new(),
        math_depth: 0,
    };

    parser.parse(html);
    parser.flush();
    parser.blocks
}

fn parse_table(html: &str) -> Option<Table> {
    let mut rows = Vec::new();
    let mut rest = html;

    while let Some(start) = find_open_tag(rest, "tr") {
        let Some(end) = find_matching_close(rest, start, "tr") else {
            break;
        };
        if let Some(row) = parse_table_row(&rest[start..end]) {
            rows.push(row);
        }
        rest = &rest[end..];
    }

    if rows.is_empty() {
        None
    } else {
        Some(Table { rows })
    }
}

fn parse_table_row(html: &str) -> Option<TableRow> {
    let mut cells = Vec::new();
    let mut rest = html;

    loop {
        let td = find_open_tag(rest, "td");
        let th = find_open_tag(rest, "th");
        let (start, name) = match (td, th) {
            (Some(td), Some(th)) if td < th => (td, "td"),
            (Some(_), Some(th)) => (th, "th"),
            (Some(td), None) => (td, "td"),
            (None, Some(th)) => (th, "th"),
            (None, None) => break,
        };

        let Some(end) = find_matching_close(rest, start, name) else {
            break;
        };
        let blocks = parse_html(&rest[start..end]);
        cells.push(TableCell {
            runs: blocks_to_runs(blocks),
            header: name == "th",
        });
        rest = &rest[end..];
    }

    if cells.is_empty() {
        None
    } else {
        Some(TableRow { cells })
    }
}

fn blocks_to_runs(blocks: Vec<Block>) -> Vec<Run> {
    let mut runs = Vec::new();

    for block in blocks {
        if !runs.is_empty() {
            runs.push(Run {
                text: "\n".into(),
                ..Run::default()
            });
        }

        match block {
            Block::Paragraph(items) | Block::Heading(_, items) => runs.extend(items),
            Block::ListItem(items) => {
                runs.push(Run {
                    text: "• ".into(),
                    ..Run::default()
                });
                runs.extend(items);
            }
            Block::Table(table) => {
                for (row_index, row) in table.rows.into_iter().enumerate() {
                    if row_index > 0 {
                        runs.push(Run {
                            text: "\n".into(),
                            ..Run::default()
                        });
                    }
                    for (cell_index, cell) in row.cells.into_iter().enumerate() {
                        if cell_index > 0 {
                            runs.push(Run {
                                text: " | ".into(),
                                ..Run::default()
                            });
                        }
                        runs.extend(cell.runs);
                    }
                }
            }
        }
    }

    runs
}

struct Parser {
    blocks: Vec<Block>,
    current: Vec<Run>,
    kind: BlockKind,
    style: Style,
    style_stack: Vec<Style>,
    link: Option<String>,
    link_stack: Vec<Option<String>>,
    math_depth: usize,
}

impl Parser {
    fn parse(&mut self, html: &str) {
        let mut rest = html;

        while let Some(start) = rest.find('<') {
            let text = &rest[..start];
            if !text.is_empty() && self.math_depth == 0 {
                self.push_text(&decode_entities(text));
            }

            let Some(end_offset) = rest[start..].find('>') else {
                break;
            };
            let end = start + end_offset;
            let tag = &rest[start + 1..end];
            self.handle_tag(tag);
            rest = &rest[end + 1..];
        }

        if !rest.is_empty() {
            self.push_text(&decode_entities(rest));
        }
    }

    fn handle_tag(&mut self, raw: &str) {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('!') {
            return;
        }

        let closing = raw.starts_with('/');
        let content = raw.trim_start_matches('/').trim_end_matches('/').trim();

        let name_end = content
            .find(|character: char| character.is_whitespace())
            .unwrap_or(content.len());
        let name = content[..name_end].to_ascii_lowercase();
        let attributes = content[name_end..].trim();

        if self.math_depth > 0 {
            if name == "span" {
                if closing {
                    self.math_depth = self.math_depth.saturating_sub(1);
                } else if !raw.ends_with('/') {
                    self.math_depth += 1;
                }
            }
            return;
        }

        match name.as_str() {
            "span" => self.span(closing, attributes, raw),
            "p" | "div" => {
                self.flush();
                if !closing {
                    self.kind = BlockKind::Paragraph;
                }
            }
            "br" => self.push_text("\n"),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                if closing {
                    self.flush();
                    self.kind = BlockKind::Paragraph;
                } else {
                    self.flush();
                    let level = name[1..].parse::<u8>().unwrap_or(1);
                    self.kind = BlockKind::Heading(level);
                }
            }
            "li" => {
                if closing {
                    self.flush();
                    self.kind = BlockKind::Paragraph;
                } else {
                    self.flush();
                    self.kind = BlockKind::ListItem;
                }
            }
            "ul" | "ol" | "blockquote" | "section" | "article" => {
                self.flush();
            }
            "strong" | "b" => self.inline(closing, |style| style.bold = true),
            "em" | "i" => self.inline(closing, |style| style.italic = true),
            "u" => self.inline(closing, |style| style.underline = true),
            "code" => self.inline(closing, |style| style.code = true),
            "a" => self.link_tag(closing, attributes),
            _ => {}
        }
    }

    fn inline(&mut self, closing: bool, set: impl FnOnce(&mut Style)) {
        if closing {
            self.style = self.style_stack.pop().unwrap_or_default();
        } else {
            self.style_stack.push(self.style.clone());
            set(&mut self.style);
        }
    }

    fn link_tag(&mut self, closing: bool, attributes: &str) {
        if closing {
            self.link = self.link_stack.pop().flatten();
            self.style = self.style_stack.pop().unwrap_or_default();
        } else {
            let href = extract_attribute(attributes, "href");
            self.link_stack.push(self.link.clone());
            self.link = href;
            self.style_stack.push(self.style.clone());
        }
    }

    fn span(&mut self, closing: bool, attributes: &str, raw: &str) {
        if closing {
            self.style = self.style_stack.pop().unwrap_or_default();
            return;
        }

        let span_style = parse_span_style(attributes);

        if attributes.contains("data-inline-math") {
            let latex =
                decode_entities(&extract_attribute(attributes, "data-latex").unwrap_or_default());
            if latex.is_empty() {
                return;
            }

            let ascii = decode_entities(
                &extract_attribute(attributes, "data-ascii").unwrap_or_else(|| latex.clone()),
            );
            let previous = self.style.clone();
            apply_style(&mut self.style, &span_style);
            self.push_math(&latex, &ascii);
            self.style = previous;

            if !raw.ends_with('/') {
                self.math_depth = 1;
            }
            return;
        }

        self.style_stack.push(self.style.clone());
        apply_style(&mut self.style, &span_style);
    }

    fn push_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.current.push(Run {
            text: text.to_string(),
            bold: self.style.bold,
            italic: self.style.italic,
            underline: self.style.underline,
            code: self.style.code,
            color: self.style.color.clone(),
            font_size: self.style.font_size,
            font_family: self.style.font_family.clone(),
            link: self.link.clone(),
            math: None,
        });
    }

    fn push_math(&mut self, latex: &str, ascii: &str) {
        self.current.push(Run {
            text: String::new(),
            bold: self.style.bold,
            italic: self.style.italic,
            underline: self.style.underline,
            code: self.style.code,
            color: self.style.color.clone(),
            font_size: self.style.font_size,
            font_family: self.style.font_family.clone(),
            link: self.link.clone(),
            math: Some(MathRun {
                latex: latex.to_string(),
                ascii: ascii.to_string(),
            }),
        });
    }

    fn flush(&mut self) {
        if self.current.is_empty() {
            return;
        }

        let runs = std::mem::take(&mut self.current);
        let block = match self.kind {
            BlockKind::Paragraph => Block::Paragraph(runs),
            BlockKind::Heading(level) => Block::Heading(level, runs),
            BlockKind::ListItem => Block::ListItem(runs),
        };
        self.blocks.push(block);
    }
}

fn find_open_tag(html: &str, name: &str) -> Option<usize> {
    let lower = html.to_ascii_lowercase();
    let needle = format!("<{name}");
    let mut offset = 0;

    while let Some(relative) = lower[offset..].find(&needle) {
        let position = offset + relative;
        let after = lower[position + needle.len()..].chars().next();
        let valid = match after {
            None | Some('>') | Some('/') => true,
            Some(character) if character.is_whitespace() => true,
            _ => false,
        };
        if valid {
            return Some(position);
        }
        offset = position + needle.len();
    }

    None
}

fn find_matching_close(html: &str, start: usize, name: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut cursor = start;

    while let Some(relative) = html[cursor..].find('<') {
        let position = cursor + relative;
        let relative_end = html[position..].find('>')?;
        let end = position + relative_end;
        let raw = html[position + 1..end].trim();
        let closing = raw.starts_with('/');
        let content = raw.trim_start_matches('/').trim_end_matches('/').trim();
        let tag_end = content
            .find(|character: char| character.is_whitespace())
            .unwrap_or(content.len());
        let tag = content[..tag_end].to_ascii_lowercase();

        if tag == name {
            if closing {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(position);
                }
            } else if !raw.ends_with('/') {
                depth += 1;
            }
        }

        cursor = end + 1;
    }

    None
}

fn apply_style(target: &mut Style, source: &Style) {
    if let Some(color) = &source.color {
        target.color = Some(color.clone());
    }
    if let Some(font_size) = source.font_size {
        target.font_size = Some(font_size);
    }
    if let Some(font_family) = &source.font_family {
        target.font_family = Some(font_family.clone());
    }
}

fn parse_span_style(attributes: &str) -> Style {
    let mut style = Style::default();
    let Some(raw) = extract_attribute(attributes, "style") else {
        return style;
    };

    for declaration in decode_entities(&raw).split(';') {
        let Some((property, value)) = declaration.split_once(':') else {
            continue;
        };

        match property.trim().to_ascii_lowercase().as_str() {
            "color" => style.color = parse_css_color(value),
            "font-size" => style.font_size = parse_css_font_size(value),
            "font-family" => {
                let family = value
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches(['"', '\'']);
                if !family.is_empty() {
                    style.font_family = Some(family.to_string());
                }
            }
            _ => {}
        }
    }

    style
}

fn parse_css_color(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();

    if let Some(hex) = value.strip_prefix('#') {
        let hex: String = match hex.len() {
            3 | 4 => {
                let digits: Vec<char> = hex.chars().take(3).collect();
                digits.iter().flat_map(|digit| [*digit, *digit]).collect()
            }
            6 | 8 => hex.chars().take(6).collect(),
            _ => return None,
        };

        if hex.chars().all(|digit| digit.is_ascii_hexdigit()) {
            return Some(format!("#{hex}"));
        }
    }

    let rgb = value
        .strip_prefix("rgb(")
        .or_else(|| value.strip_prefix("rgba("))
        .and_then(|inner| inner.strip_suffix(')'));
    if let Some(inner) = rgb {
        let channels: Vec<&str> = inner
            .split([',', ' ', '/'])
            .filter(|channel| !channel.is_empty())
            .take(3)
            .collect();
        if channels.len() == 3 {
            let mut bytes = [0u8; 3];
            for (index, channel) in channels.iter().enumerate() {
                let amount = if let Some(percent) = channel.strip_suffix('%') {
                    percent.parse::<f64>().ok()? * 255.0 / 100.0
                } else {
                    channel.parse::<f64>().ok()?
                };
                if !amount.is_finite() {
                    return None;
                }
                bytes[index] = amount.round().clamp(0.0, 255.0) as u8;
            }
            return Some(format!("#{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2]));
        }
    }

    named_color(&value).map(|color| color.to_string())
}

fn named_color(value: &str) -> Option<&'static str> {
    match value {
        "black" => Some("#000000"),
        "white" => Some("#ffffff"),
        "red" => Some("#ff0000"),
        "green" => Some("#008000"),
        "blue" => Some("#0000ff"),
        "yellow" => Some("#ffff00"),
        "orange" => Some("#ffa500"),
        "purple" => Some("#800080"),
        "gray" | "grey" => Some("#808080"),
        "silver" => Some("#c0c0c0"),
        "maroon" => Some("#800000"),
        "navy" => Some("#000080"),
        "teal" => Some("#008080"),
        "lime" => Some("#00ff00"),
        "aqua" => Some("#00ffff"),
        "fuchsia" => Some("#ff00ff"),
        _ => None,
    }
}

fn parse_css_font_size(value: &str) -> Option<f64> {
    let value = value.trim().to_ascii_lowercase();
    let unit_start =
        value.find(|character: char| !(character.is_ascii_digit() || character == '.'))?;
    let amount = value[..unit_start].parse::<f64>().ok()?;
    let pixels = match value[unit_start..].trim() {
        "px" => amount,
        "pt" => amount * 96.0 / 72.0,
        "em" | "rem" => amount * 16.0,
        "%" => amount * 16.0 / 100.0,
        _ => return None,
    };

    if !pixels.is_finite() {
        return None;
    }

    Some(pixels.clamp(6.0, 96.0))
}

fn extract_attribute(attributes: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=");
    let start = attributes.to_ascii_lowercase().find(needle.as_str())?;
    let value = &attributes[start + needle.len()..];
    let value = value.trim_start();

    if let Some(rest) = value.strip_prefix('"') {
        return rest.split('"').next().map(|value| value.to_string());
    }
    if let Some(rest) = value.strip_prefix('\'') {
        return rest.split('\'').next().map(|value| value.to_string());
    }

    value
        .split_whitespace()
        .next()
        .map(|value| value.to_string())
}

fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_paragraph_and_bold() {
        let blocks = parse_html("<p>Hello <strong>world</strong></p>");
        assert_eq!(blocks.len(), 1);
        let Block::Paragraph(runs) = &blocks[0] else {
            panic!("expected a paragraph");
        };
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].text, "Hello ");
        assert!(runs[1].bold);
        assert_eq!(runs[1].text, "world");
    }

    #[test]
    fn parses_heading_and_list() {
        let blocks = parse_html("<h2>Title</h2><ul><li>One</li><li>Two</li></ul>");
        assert!(matches!(blocks.first(), Some(Block::Heading(2, _))));
        assert!(matches!(blocks.get(1), Some(Block::ListItem(_))));
        assert!(matches!(blocks.get(2), Some(Block::ListItem(_))));
    }

    #[test]
    fn parses_link() {
        let blocks = parse_html(r#"<p><a href="https://example.com">Ejemplo</a></p>"#);
        let Block::Paragraph(runs) = &blocks[0] else {
            panic!("expected a paragraph");
        };
        assert_eq!(runs[0].text, "Ejemplo");
        assert_eq!(runs[0].link.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn parses_table_with_header_and_formatting() {
        let blocks = parse_html(
            r#"<table><tbody><tr><th><p>Nombre</p></th><th><p>Valor</p></th></tr><tr><td><p><strong>Uno</strong></p></td><td><p><span style="color: #ff0000">2</span></p></td></tr></tbody></table>"#,
        );
        let Block::Table(table) = &blocks[0] else {
            panic!("expected a table");
        };
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells.len(), 2);
        assert!(table.rows[0].cells[0].header);
        assert_eq!(table.rows[0].cells[0].runs[0].text, "Nombre");
        assert!(table.rows[1].cells[0].runs[0].bold);
        assert_eq!(
            table.rows[1].cells[1].runs[0].color.as_deref(),
            Some("#ff0000")
        );
    }

    #[test]
    fn parses_color_and_font_size() {
        let blocks = parse_html(
            r#"<p><span style="color: rgb(255, 0, 0); font-size: 20px">Rojo</span></p>"#,
        );
        let Block::Paragraph(runs) = &blocks[0] else {
            panic!("expected a paragraph");
        };
        assert_eq!(runs[0].text, "Rojo");
        assert_eq!(runs[0].color.as_deref(), Some("#ff0000"));
        assert_eq!(runs[0].font_size, Some(20.0));
    }

    #[test]
    fn parses_inline_math() {
        let blocks = parse_html(
            "<p>Area: <span data-latex=\"x^2\" data-ascii=\"x^2\" data-inline-math=\"\"></span> unidades</p>",
        );
        let Block::Paragraph(runs) = &blocks[0] else {
            panic!("expected a paragraph");
        };

        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].text, "Area: ");
        let math = runs[1].math.as_ref().expect("inline math run");
        assert_eq!(math.latex, "x^2");
        assert_eq!(math.ascii, "x^2");
        assert_eq!(runs[2].text, " unidades");
    }
}
