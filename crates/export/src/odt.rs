// SPDX-License-Identifier: GPL-3.0-or-later

use base64::Engine as _;
use doc_core::{Document, Region};
use std::fmt::Write as _;
use std::fs;
use std::io::{Cursor, Write as _};
use std::path::PathBuf;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::html::{parse_html, plain_text, Block, Run, Table};
use crate::latex::{extension_to_mime, mime_to_extension};
use crate::mathml::{mathml_for, xml_escape, MathMl};
use crate::results::evaluate_math_results;
use crate::ExportError;

const MIMETYPE: &str = "application/vnd.oasis.opendocument.text";

pub fn export_odt(document: &Document) -> Result<Vec<u8>, ExportError> {
    let mut body = String::new();
    let mut text_styles = TextStyles::default();
    let mut table_index = 0usize;
    let mut images: Vec<(String, Vec<u8>, String)> = Vec::new();
    let mut image_index = 0usize;
    let mut formula_index = 0usize;
    let mut formula_objects: Vec<(String, String)> = Vec::new();
    let math_results = evaluate_math_results(document);

    for (page_index, page) in document.pages.iter().enumerate() {
        if page_index > 0 {
            body.push_str("<text:p text:style-name=\"PageBreak\"/>");
        }

        for region in &page.regions {
            match region {
                Region::Text(text) => {
                    if text
                        .role
                        .as_deref()
                        .is_some_and(|role| role == "header" || role == "footer")
                    {
                        continue;
                    }
                    body.push_str(&blocks_to_odt(
                        &parse_html(&text.html),
                        &mut text_styles,
                        &mut table_index,
                    ));
                }
                Region::Math(math) => match mathml_for(&math.ascii, &math.source) {
                    MathMl::Formula(formula) => {
                        formula_index += 1;
                        let object_name = format!("Object {formula_index}");
                        let object_content = formula.replacen(
                            "<math:math>",
                            "<math:math xmlns:math=\"http://www.w3.org/1998/Math/MathML\">",
                            1,
                        );
                        formula_objects.push((
                            object_name.clone(),
                            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{object_content}"),
                        ));
                        let source = if math.source.trim().is_empty() {
                            &math.ascii
                        } else {
                            &math.source
                        };
                        let matrix_tables = formula
                            .split("<math:mtable>")
                            .skip(1)
                            .map(|table| table.split("</math:mtable>").next().unwrap_or(""))
                            .collect::<Vec<_>>();
                        let matrix_rows = matrix_tables
                            .iter()
                            .map(|table| table.matches("<math:mtr>").count())
                            .max()
                            .unwrap_or(0);
                        let matrix_columns = matrix_tables
                            .iter()
                            .flat_map(|table| table.split("<math:mtr>").skip(1))
                            .map(|row| {
                                row.split("</math:mtr>")
                                    .next()
                                    .unwrap_or("")
                                    .matches("<math:mtd>")
                                    .count()
                            })
                            .max()
                            .unwrap_or(0);
                        let matrix = !matrix_tables.is_empty() && matrix_columns > 0;
                        let prefix_chars = source
                            .find('[')
                            .or_else(|| source.find("\\begin"))
                            .or_else(|| source.find('('))
                            .unwrap_or(0);
                        let width = if matrix {
                            (0.9 + prefix_chars as f64 * 0.14 + matrix_columns as f64 * 0.65)
                                .clamp(2.2, 12.0)
                        } else {
                            (0.9 + source.chars().count() as f64 * 0.16).clamp(1.2, 14.0)
                        };
                        let height = if matrix {
                            (0.55 + matrix_rows as f64 * 0.55).clamp(1.1, 4.5)
                        } else if formula.contains("<math:mfrac>")
                            || formula.contains("<math:msqrt>")
                            || formula.contains("<math:mroot>")
                        {
                            1.25
                        } else if formula.contains("<math:msup>") {
                            0.75
                        } else {
                            0.65
                        };
                        write!(
                                body,
                                "<text:p text:style-name=\"P1\"><draw:frame draw:style-name=\"fr1\" draw:name=\"Formula{formula_index}\" text:anchor-type=\"as-char\" svg:width=\"{width:.2}cm\" svg:height=\"{height:.2}cm\"><draw:object xlink:href=\"./{object_name}\" xlink:type=\"simple\" xlink:show=\"embed\" xlink:actuate=\"onLoad\"/></draw:frame>"
                            )
                            .expect("writing to a String cannot fail");
                        if let Some(result) = math_results.get(&math.id) {
                            let escaped = xml_escape(result);
                            if source.trim_end().ends_with('=') {
                                write!(
                                    body,
                                    " = <text:span text:style-name=\"ResultInline\">{escaped}</text:span>"
                                )
                                .expect("writing to a String cannot fail");
                            } else {
                                write!(
                                    body,
                                    " <text:span text:style-name=\"Result\">= {escaped}</text:span>"
                                )
                                .expect("writing to a String cannot fail");
                            }
                        }
                        body.push_str("</text:p>");
                    }
                    MathMl::Fallback(fallback) => {
                        body.push_str("<text:p text:style-name=\"P1\">");
                        body.push_str(&fallback);
                        body.push_str("</text:p>");
                    }
                },
                Region::Image(image) => {
                    image_index += 1;
                    if let Some((name, bytes, mime)) = read_image(&image.src, image_index)? {
                        let width = format!("{:.2}", image.w);
                        let height = format!("{:.2}", image.h);
                        write!(
                            body,
                            "<text:p text:style-name=\"P1\"><draw:frame draw:name=\"Image{image_index}\" text:anchor-type=\"as-char\" svg:width=\"{width}pt\" svg:height=\"{height}pt\"><draw:image xlink:href=\"Pictures/{name}\" xlink:type=\"simple\" xlink:show=\"embed\" xlink:actuate=\"onLoad\"/></draw:frame></text:p>"
                        )
                        .expect("writing to a String cannot fail");
                        images.push((name, bytes, mime));
                    }
                }
                Region::Plot(plot) => {
                    let expressions = plot
                        .series
                        .iter()
                        .filter(|series| series.visible)
                        .map(|series| xml_escape(&series.expression))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(
                        body,
                        "<text:p text:style-name=\"P1\">Gráfico 2D: {expressions}</text:p>"
                    )
                    .expect("writing to a String cannot fail");
                }
            }
        }
    }

    let content = render_content(&body, &text_styles);
    let styles = render_styles(document);
    let meta = render_meta(document);
    let settings = render_settings();
    let manifest = render_manifest(&images, &formula_objects);

    let cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(cursor);

    zip.start_file("mimetype", stored_options())?;
    zip.write_all(MIMETYPE.as_bytes())?;

    write_zip_file(&mut zip, "content.xml", content.as_bytes())?;
    write_zip_file(&mut zip, "styles.xml", styles.as_bytes())?;
    write_zip_file(&mut zip, "meta.xml", meta.as_bytes())?;
    write_zip_file(&mut zip, "settings.xml", settings.as_bytes())?;
    write_zip_file(&mut zip, "META-INF/manifest.xml", manifest.as_bytes())?;

    for (name, bytes, _) in &images {
        write_zip_file(&mut zip, &format!("Pictures/{name}"), bytes)?;
    }

    for (name, content) in &formula_objects {
        write_zip_file(&mut zip, &format!("{name}/content.xml"), content.as_bytes())?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

fn write_zip_file(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &str,
    contents: &[u8],
) -> Result<(), ExportError> {
    zip.start_file(name, deflated_options())?;
    zip.write_all(contents)?;
    Ok(())
}

fn stored_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
}

fn deflated_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
}

fn blocks_to_odt(
    blocks: &[Block],
    text_styles: &mut TextStyles,
    table_index: &mut usize,
) -> String {
    let mut odt = String::new();
    let mut in_list = false;

    for block in blocks {
        if !matches!(block, Block::ListItem(_)) && in_list {
            odt.push_str("</text:list>");
            in_list = false;
        }

        match block {
            Block::Paragraph(runs) => {
                odt.push_str("<text:p text:style-name=\"P1\">");
                odt.push_str(&runs_to_odt(runs, text_styles));
                odt.push_str("</text:p>");
            }
            Block::Heading(level, runs) => {
                write!(odt, "<text:h text:outline-level=\"{level}\">")
                    .expect("writing to a String cannot fail");
                odt.push_str(&runs_to_odt(runs, text_styles));
                odt.push_str("</text:h>");
            }
            Block::ListItem(runs) => {
                if !in_list {
                    odt.push_str("<text:list>");
                    in_list = true;
                }
                odt.push_str("<text:list-item><text:p text:style-name=\"P1\">");
                odt.push_str(&runs_to_odt(runs, text_styles));
                odt.push_str("</text:p></text:list-item>");
            }
            Block::Table(table) => {
                *table_index += 1;
                odt.push_str(&table_to_odt(table, *table_index, text_styles));
            }
        }
    }

    if in_list {
        odt.push_str("</text:list>");
    }

    odt
}

fn table_to_odt(table: &Table, index: usize, text_styles: &mut TextStyles) -> String {
    let columns = table
        .rows
        .iter()
        .map(|row| row.cells.len())
        .max()
        .unwrap_or(0);

    if columns == 0 {
        return String::new();
    }

    let mut odt = format!(
        "<table:table table:name=\"Table{index}\" table:style-name=\"Table\"><table:table-column table:number-columns-repeated=\"{columns}\" table:style-name=\"TableColumn\"/>"
    );

    for row in &table.rows {
        odt.push_str("<table:table-row>");
        for column in 0..columns {
            let cell = row.cells.get(column);
            let header = cell.is_some_and(|cell| cell.header);
            let cell_style = if header {
                "TableHeaderCell"
            } else {
                "TableCell"
            };
            let paragraph_style = if header { "TableHeader" } else { "P1" };

            write!(
                odt,
                "<table:table-cell table:style-name=\"{cell_style}\" office:value-type=\"string\"><text:p text:style-name=\"{paragraph_style}\">"
            )
            .expect("writing to a String cannot fail");
            if let Some(cell) = cell {
                odt.push_str(&runs_to_odt(&cell.runs, text_styles));
            }
            odt.push_str("</text:p></table:table-cell>");
        }
        odt.push_str("</table:table-row>");
    }

    odt.push_str("</table:table>");
    odt
}

fn runs_to_odt(runs: &[Run], text_styles: &mut TextStyles) -> String {
    runs.iter()
        .map(|run| run_to_odt(run, text_styles))
        .collect::<Vec<_>>()
        .join("")
}

fn run_to_odt(run: &Run, text_styles: &mut TextStyles) -> String {
    let mut text = if let Some(math) = &run.math {
        let text = if math.ascii.is_empty() {
            &math.latex
        } else {
            &math.ascii
        };
        format!(
            "<text:span text:style-name=\"MathFallback\">{}</text:span>",
            xml_escape(text)
        )
    } else {
        xml_escape(&run.text).replace('\n', "<text:line-break/>")
    };

    if run.code {
        text = format!("<text:span text:style-name=\"Code\">{text}</text:span>");
    }
    if run.bold {
        text = format!("<text:span text:style-name=\"Strong\">{text}</text:span>");
    }
    if run.italic {
        text = format!("<text:span text:style-name=\"Emphasis\">{text}</text:span>");
    }
    if run.underline {
        text = format!("<text:span text:style-name=\"Underline\">{text}</text:span>");
    }
    if let Some(link) = &run.link {
        text = format!(
            "<text:a xlink:type=\"simple\" xlink:href=\"{}\">{text}</text:a>",
            xml_escape(link)
        );
    }
    if let Some(style) = text_styles.style_name(
        run.color.as_deref(),
        run.font_size,
        run.font_family.as_deref(),
    ) {
        text = format!("<text:span text:style-name=\"{style}\">{text}</text:span>");
    }

    text
}

#[derive(Default)]
struct TextStyles {
    entries: Vec<TextStyleEntry>,
}

struct TextStyleEntry {
    name: String,
    key: String,
    properties: String,
}

impl TextStyles {
    fn style_name(
        &mut self,
        color: Option<&str>,
        font_size: Option<f64>,
        font_family: Option<&str>,
    ) -> Option<String> {
        if color.is_none() && font_size.is_none() && font_family.is_none() {
            return None;
        }

        let key = format!(
            "{}|{}|{}",
            color.unwrap_or_default(),
            font_size
                .map(|size| format!("{size:.2}"))
                .unwrap_or_default(),
            font_family.unwrap_or_default()
        );

        if let Some(entry) = self.entries.iter().find(|entry| entry.key == key) {
            return Some(entry.name.clone());
        }

        let name = format!("Txt{}", self.entries.len() + 1);
        let mut properties = String::new();
        if let Some(color) = color {
            write!(properties, " fo:color=\"{}\"", xml_escape(color))
                .expect("writing to a String cannot fail");
        }
        if let Some(font_size) = font_size {
            write!(properties, " fo:font-size=\"{:.1}pt\"", font_size * 0.75)
                .expect("writing to a String cannot fail");
        }
        if let Some(font_family) = font_family {
            write!(
                properties,
                " fo:font-family=\"{}\"",
                xml_escape(font_family)
            )
            .expect("writing to a String cannot fail");
        }

        self.entries.push(TextStyleEntry {
            name: name.clone(),
            key,
            properties,
        });
        Some(name)
    }

    fn to_xml(&self) -> String {
        self.entries
            .iter()
            .map(|entry| {
                format!(
                    "   <style:style style:name=\"{}\" style:family=\"text\"><style:text-properties{}/></style:style>\n",
                    entry.name, entry.properties
                )
            })
            .collect()
    }
}

fn render_content(body: &str, text_styles: &TextStyles) -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content
 xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"
 xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 xmlns:xlink="http://www.w3.org/1999/xlink"
 xmlns:dc="http://purl.org/dc/elements/1.1/"
 xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"
 xmlns:number="urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0"
 xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
 xmlns:math="http://www.w3.org/1998/Math/MathML"
 office:version="1.3">
 <office:automatic-styles>
  <style:style style:name="P1" style:family="paragraph" style:parent-style-name="Standard"/>
  <style:style style:name="PageBreak" style:family="paragraph" style:parent-style-name="Standard"><style:paragraph-properties fo:break-before="page"/></style:style>
  <style:style style:name="fr1" style:family="graphic" style:parent-style-name="Graphics"><style:graphic-properties draw:fill="none" draw:stroke="none" fo:border="none" style:wrap="none"/></style:style>
  <style:style style:name="Strong" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
  <style:style style:name="Result" style:family="text"><style:text-properties fo:color="#047857" fo:font-size="10pt"/></style:style>
  <style:style style:name="ResultInline" style:family="text"><style:text-properties fo:color="#111827" fo:font-weight="bold" fo:font-size="12pt"/></style:style>
  <style:style style:name="Emphasis" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
  <style:style style:name="Underline" style:family="text"><style:text-properties style:text-underline-style="solid" style:text-underline-width="auto" style:text-underline-color="font-color"/></style:style>
  <style:style style:name="Code" style:family="text"><style:text-properties fo:font-family="monospace"/></style:style>
  <style:style style:name="MathFallback" style:family="text"><style:text-properties fo:font-family="monospace"/></style:style>
  <style:style style:name="Table" style:family="table"><style:table-properties table:border-model="collapsing" fo:margin-top="0.2cm" fo:margin-bottom="0.2cm"/></style:style>
  <style:style style:name="TableColumn" style:family="table-column"><style:table-column-properties style:column-width="3.5cm"/></style:style>
  <style:style style:name="TableCell" style:family="table-cell"><style:table-cell-properties fo:padding="0.1cm" fo:border="0.5pt solid #cbd5e1"/></style:style>
  <style:style style:name="TableHeaderCell" style:family="table-cell"><style:table-cell-properties fo:padding="0.1cm" fo:border="0.5pt solid #94a3b8" fo:background-color="#eef2f7"/></style:style>
  <style:style style:name="TableHeader" style:family="paragraph" style:parent-style-name="P1"><style:text-properties fo:font-weight="bold"/></style:style>
{text_styles} </office:automatic-styles>
 <office:body>
  <office:text>{body}</office:text>
 </office:body>
</office:document-content>
"##,
        text_styles = text_styles.to_xml()
    )
}

fn render_styles(document: &Document) -> String {
    let setup = &document.settings.page;
    let (width, height) = document
        .pages
        .first()
        .map(|page| (page.width, page.height))
        .unwrap_or((794.0, 1123.0));
    let orientation = if width >= height {
        "landscape"
    } else {
        "portrait"
    };
    let header =
        running_text_from_regions(document, "header").unwrap_or_else(|| setup.header.clone());
    let footer =
        running_text_from_regions(document, "footer").unwrap_or_else(|| setup.footer.clone());
    let has_header = !header.trim().is_empty();
    let has_footer = !footer.trim().is_empty();

    let mut styles = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles
 xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 office:version="1.3">
 <office:font-face-decls/>
 <office:styles>
  <style:default-style style:family="paragraph">
   <style:paragraph-properties fo:margin-top="0cm" fo:margin-bottom="0.2cm"/>
  </style:default-style>
  <style:style style:name="Standard" style:family="paragraph" style:class="text"/>
  <style:style style:name="Header" style:family="paragraph">
   <style:paragraph-properties fo:text-align="start" fo:margin-top="0cm" fo:margin-bottom="0cm"/>
  </style:style>
  <style:style style:name="Footer" style:family="paragraph">
   <style:paragraph-properties fo:text-align="end" fo:margin-top="0cm" fo:margin-bottom="0cm"/>
  </style:style>
 </office:styles>
 <office:automatic-styles>
  <style:page-layout style:name="pm1">
   <style:page-layout-properties fo:page-width="{}cm" fo:page-height="{}cm" style:print-orientation="{}" fo:margin-top="{}cm" fo:margin-right="{}cm" fo:margin-bottom="{}cm" fo:margin-left="{}cm"/>
"#,
        px_to_cm(width),
        px_to_cm(height),
        orientation,
        px_to_cm(setup.margin_top),
        px_to_cm(setup.margin_right),
        px_to_cm(setup.margin_bottom),
        px_to_cm(setup.margin_left)
    );

    if has_header {
        styles.push_str(
            "   <style:header-style><style:header-footer-properties fo:margin-bottom=\"0.3cm\" fo:min-height=\"0.5cm\"/></style:header-style>\n",
        );
    }
    if has_footer {
        styles.push_str(
            "   <style:footer-style><style:header-footer-properties fo:margin-top=\"0.3cm\" fo:min-height=\"0.5cm\"/></style:footer-style>\n",
        );
    }

    styles.push_str("  </style:page-layout>\n </office:automatic-styles>\n");
    styles.push_str(" <office:master-styles>\n");
    styles
        .push_str("  <style:master-page style:name=\"Standard\" style:page-layout-name=\"pm1\">\n");

    if has_header {
        writeln!(
            styles,
            "   <style:header><text:p text:style-name=\"Header\">{}</text:p></style:header>",
            running_text_odt(&header)
        )
        .expect("writing to a String cannot fail");
    }
    if has_footer {
        writeln!(
            styles,
            "   <style:footer><text:p text:style-name=\"Footer\">{}</text:p></style:footer>",
            running_text_odt(&footer)
        )
        .expect("writing to a String cannot fail");
    }

    styles.push_str("  </style:master-page>\n </office:master-styles>\n");
    styles.push_str("</office:document-styles>\n");
    styles
}

fn px_to_cm(value: f64) -> String {
    let centimeters = value.max(0.0) / 96.0 * 2.54;
    format!("{:.2}", (centimeters * 100.0).round() / 100.0)
}

fn running_text_odt(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;

    loop {
        let lower = rest.to_ascii_lowercase();
        if let Some(index) = lower.find("{page}") {
            result.push_str(&xml_escape(&rest[..index]));
            result.push_str("<text:page-number text:select-page=\"current\">1</text:page-number>");
            rest = &rest[index + "{page}".len()..];
        } else {
            result.push_str(&xml_escape(rest));
            break;
        }
    }

    result
}

fn render_meta(document: &Document) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta
 xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"
 xmlns:dc="http://purl.org/dc/elements/1.1/"
 office:version="1.3">
 <office:meta>
  <dc:title>{}</dc:title>
  <dc:creator>{}</dc:creator>
 </office:meta>
</office:document-meta>
"#,
        xml_escape(&document.metadata.title),
        xml_escape(&document.metadata.author)
    )
}

fn render_settings() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-settings
 xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:config="urn:oasis:names:tc:opendocument:xmlns:config:1.0"
 office:version="1.3">
 <office:settings/>
</office:document-settings>
"#
    .to_string()
}

fn render_manifest(images: &[(String, Vec<u8>, String)], formulas: &[(String, String)]) -> String {
    let mut manifest = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest
 xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"
 manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="settings.xml" manifest:media-type="text/xml"/>
"#,
    );

    for (name, _) in formulas {
        writeln!(
            manifest,
            " <manifest:file-entry manifest:full-path=\"{name}/\" manifest:media-type=\"application/vnd.oasis.opendocument.formula\"/>"
        )
        .expect("writing to a String cannot fail");
        writeln!(
            manifest,
            " <manifest:file-entry manifest:full-path=\"{name}/content.xml\" manifest:media-type=\"text/xml\"/>"
        )
        .expect("writing to a String cannot fail");
    }

    for (name, _, mime) in images {
        writeln!(
            manifest,
            " <manifest:file-entry manifest:full-path=\"Pictures/{name}\" manifest:media-type=\"{}\"/>",
            xml_escape(mime)
        )
        .expect("writing to a String cannot fail");
    }

    manifest.push_str("</manifest:manifest>\n");
    manifest
}

fn read_image(
    source: &str,
    index: usize,
) -> Result<Option<(String, Vec<u8>, String)>, ExportError> {
    if let Some(data) = source.strip_prefix("data:") {
        let Some((metadata, payload)) = data.split_once(',') else {
            return Ok(None);
        };
        if !metadata.contains(";base64") {
            return Ok(None);
        }
        let mime = metadata
            .split(';')
            .next()
            .unwrap_or("application/octet-stream");
        let extension = mime_to_extension(mime).unwrap_or("bin");
        let bytes = base64::engine::general_purpose::STANDARD.decode(payload)?;
        return Ok(Some((
            format!("image_{index}.{extension}"),
            bytes,
            mime.to_string(),
        )));
    }

    let path = PathBuf::from(source);
    if path.exists() {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("bin");
        let mime = extension_to_mime(extension);
        return Ok(Some((
            format!("image_{index}.{extension}"),
            fs::read(path)?,
            mime.to_string(),
        )));
    }

    Ok(None)
}

fn running_text_from_regions(document: &Document, role: &str) -> Option<String> {
    document
        .pages
        .iter()
        .flat_map(|page| page.regions.iter())
        .find_map(|region| match region {
            Region::Text(text) if text.role.as_deref() == Some(role) => {
                Some(plain_text(&text.html))
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_core::TextRegion;
    use std::io::Read;

    #[test]
    fn renders_minimal_content() {
        let document = Document::default();
        let bytes = export_odt(&document).expect("ODT should be generated");
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("ODT should be a readable zip archive");

        let mut content = String::new();
        archive
            .by_name("content.xml")
            .expect("content.xml")
            .read_to_string(&mut content)
            .expect("content.xml should be UTF-8");
        assert!(content.contains("<office:document-content"));
        assert!(content.contains("<draw:object"));
        assert!(content.contains("xlink:href=\"./Object 1\""));
        assert!(content.contains("fo:border=\"none\""));
        assert!(content.contains("text:style-name=\"Result\""));
        assert!(content.contains("= 4"));

        let mut formula = String::new();
        archive
            .by_name("Object 1/content.xml")
            .expect("Object 1/content.xml")
            .read_to_string(&mut formula)
            .expect("formula object should be UTF-8");
        assert!(formula.contains("<math:math"));
        assert!(formula.contains("xmlns:math="));
        assert!(formula.contains("2+2"));

        let mut manifest = String::new();
        archive
            .by_name("META-INF/manifest.xml")
            .expect("manifest")
            .read_to_string(&mut manifest)
            .expect("manifest should be UTF-8");
        assert!(manifest.contains("application/vnd.oasis.opendocument.text"));
        assert!(manifest.contains("Object 1/"));
    }

    #[test]
    fn sizes_matrix_formulas_without_distortion() {
        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Math(doc_core::MathRegion {
            id: "math-matrix".into(),
            x: 60.0,
            y: 60.0,
            w: 360.0,
            h: 48.0,
            source: "A := \\begin{pmatrix}1 & 2\\\\3 & 4\\end{pmatrix}".into(),
            ascii: "A := [1,2;3,4]".into(),
            pinned: false,
        })];

        let bytes = export_odt(&document).expect("ODT should be generated");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("ODT should be a readable zip archive");
        let mut content = String::new();
        archive
            .by_name("content.xml")
            .expect("content.xml")
            .read_to_string(&mut content)
            .expect("content.xml should be UTF-8");

        assert!(content.contains("svg:width=\"2.90cm\""));
        assert!(content.contains("svg:height=\"1.65cm\""));

        let mut formula = String::new();
        archive
            .by_name("Object 1/content.xml")
            .expect("Object 1/content.xml")
            .read_to_string(&mut formula)
            .expect("formula object should be UTF-8");
        assert!(formula.contains("<math:mtable>"));
        assert!(formula.contains("<math:mtr>"));
        assert_eq!(formula.matches("<math:mtd>").count(), 4);
    }

    #[test]
    fn exports_inline_result_for_trailing_equals() {
        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Math(doc_core::MathRegion {
            id: "math-result".into(),
            x: 60.0,
            y: 60.0,
            w: 360.0,
            h: 48.0,
            source: "2+2=".into(),
            ascii: "2+2=".into(),
            pinned: false,
        })];

        let bytes = export_odt(&document).expect("ODT should be generated");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("ODT should be a readable zip archive");
        let mut content = String::new();
        archive
            .by_name("content.xml")
            .expect("content.xml")
            .read_to_string(&mut content)
            .expect("content.xml should be UTF-8");

        assert!(content.contains("text:style-name=\"ResultInline\""));
        assert!(content.contains(">4</text:span>"));
    }

    #[test]
    fn renders_table_with_header() {
        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Text(TextRegion {
            id: "text-table".into(),
            x: 60.0,
            y: 60.0,
            w: 500.0,
            h: 160.0,
            html: r#"<table><tbody><tr><th><p>Nombre</p></th><th><p>Valor</p></th></tr><tr><td><p>Uno</p></td><td><p>2</p></td></tr></tbody></table>"#.into(),
            role: None,
            pinned: false,
        })];

        let bytes = export_odt(&document).expect("ODT should be generated");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("ODT should be a readable zip archive");
        let mut content = String::new();
        archive
            .by_name("content.xml")
            .expect("content.xml")
            .read_to_string(&mut content)
            .expect("content.xml should be UTF-8");

        assert!(content.contains("<table:table table:name=\"Table1\""));
        assert!(content.contains("table:style-name=\"TableHeaderCell\""));
        assert!(content.contains("<text:p text:style-name=\"TableHeader\">Nombre</text:p>"));
        assert!(content.contains("<text:p text:style-name=\"P1\">Uno</text:p>"));

        let mut styles = String::new();
        archive
            .by_name("styles.xml")
            .expect("styles.xml")
            .read_to_string(&mut styles)
            .expect("styles.xml should be UTF-8");
        assert!(styles.contains("style:page-layout"));
    }

    #[test]
    fn renders_text_color_and_font_size() {
        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Text(TextRegion {
            id: "text-color".into(),
            x: 60.0,
            y: 60.0,
            w: 500.0,
            h: 100.0,
            html: r#"<p><span style="color: #ff0000; font-size: 20px; font-family: Arial">Rojo</span></p>"#.into(),
            role: None,
            pinned: false,
        })];

        let bytes = export_odt(&document).expect("ODT should be generated");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("ODT should be a readable zip archive");
        let mut content = String::new();
        archive
            .by_name("content.xml")
            .expect("content.xml")
            .read_to_string(&mut content)
            .expect("content.xml should be UTF-8");

        assert!(content.contains("fo:color=\"#ff0000\""));
        assert!(content.contains("fo:font-size=\"15.0pt\""));
        assert!(content.contains("fo:font-family=\"Arial\""));
        assert!(content.contains("text:span text:style-name=\"Txt1\""));
    }

    #[test]
    fn renders_page_layout_and_running_text() {
        let mut document = Document::default();
        document.settings.page.header = "Informe {page}".into();
        document.settings.page.footer = "Página {page}".into();
        document.settings.page.margin_top = 96.0;
        document.settings.page.margin_left = 60.0;
        document.pages[0].width = 794.0;
        document.pages[0].height = 1123.0;

        let bytes = export_odt(&document).expect("ODT should be generated");
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .expect("ODT should be a readable zip archive");
        let mut styles = String::new();
        archive
            .by_name("styles.xml")
            .expect("styles.xml")
            .read_to_string(&mut styles)
            .expect("styles.xml should be UTF-8");

        assert!(styles.contains("fo:page-width=\"21.01cm\""));
        assert!(styles.contains("fo:page-height=\"29.71cm\""));
        assert!(styles.contains("fo:margin-top=\"2.54cm\""));
        assert!(styles.contains("fo:margin-left=\"1.59cm\""));
        assert!(styles.contains("<style:header>"));
        assert!(styles.contains("Informe <text:page-number"));
        assert!(styles.contains("<style:footer>"));
        assert!(styles.contains("Página <text:page-number"));
        assert!(!styles.contains("style:master-page-name=\"Standard\""));
    }
}
