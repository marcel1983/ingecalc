// SPDX-License-Identifier: GPL-3.0-or-later

use base64::Engine as _;
use doc_core::{Document, Region};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::html::{parse_html, plain_text, Block, Run};
use crate::results::evaluate_math_results;
use crate::ExportError;

pub fn export_latex(document: &Document, output: &Path) -> Result<(), ExportError> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let assets_dir = output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("assets");
    let mut image_index = 0usize;
    let mut latex = String::new();

    latex.push_str("\\documentclass[11pt]{article}\n");
    latex.push_str("\\usepackage{iftex}\n");
    latex.push_str("\\usepackage{amsmath}\n");
    latex.push_str("\\ifPDFTeX\n");
    latex.push_str("  \\usepackage[utf8]{inputenc}\n");
    latex.push_str("  \\usepackage[T1]{fontenc}\n");
    latex.push_str("  \\usepackage{amssymb}\n");
    latex.push_str("\\else\n");
    latex.push_str("  \\usepackage{fontspec}\n");
    latex.push_str("  \\usepackage{unicode-math}\n");
    latex.push_str("\\fi\n");
    latex.push_str("\\usepackage{textcomp}\n");
    latex.push_str("\\usepackage{graphicx}\n");

    let (page_width, page_height) = document
        .pages
        .first()
        .map(|page| (page.width, page.height))
        .unwrap_or((794.0, 1123.0));
    let setup = &document.settings.page;
    let math_results = evaluate_math_results(document);
    writeln!(
        latex,
        "\\usepackage[paperwidth={}cm,paperheight={}cm,top={}cm,right={}cm,bottom={}cm,left={}cm]{{geometry}}",
        px_to_cm(page_width),
        px_to_cm(page_height),
        px_to_cm(setup.margin_top),
        px_to_cm(setup.margin_right),
        px_to_cm(setup.margin_bottom),
        px_to_cm(setup.margin_left)
    )
    .expect("writing to a String cannot fail");

    let header =
        running_text_from_regions(document, "header").unwrap_or_else(|| setup.header.clone());
    let footer =
        running_text_from_regions(document, "footer").unwrap_or_else(|| setup.footer.clone());
    let has_running_text = !header.trim().is_empty() || !footer.trim().is_empty();
    if has_running_text {
        latex.push_str("\\makeatletter\n");
        latex.push_str("\\newcommand{\\ps@ingecalc}{%\n");
        writeln!(
            latex,
            "  \\renewcommand{{\\@oddhead}}{{\\mbox{{{}}}\\hfil}}%",
            running_text_to_latex(&header)
        )
        .expect("writing to a String cannot fail");
        writeln!(
            latex,
            "  \\renewcommand{{\\@evenhead}}{{\\mbox{{{}}}\\hfil}}%",
            running_text_to_latex(&header)
        )
        .expect("writing to a String cannot fail");
        writeln!(
            latex,
            "  \\renewcommand{{\\@oddfoot}}{{\\hfil\\mbox{{{}}}}}%",
            running_text_to_latex(&footer)
        )
        .expect("writing to a String cannot fail");
        writeln!(
            latex,
            "  \\renewcommand{{\\@evenfoot}}{{\\hfil\\mbox{{{}}}}}%",
            running_text_to_latex(&footer)
        )
        .expect("writing to a String cannot fail");
        latex.push_str("}\n");
        latex.push_str("\\makeatother\n");
        latex.push_str("\\pagestyle{ingecalc}\n");
    }

    latex.push_str("\\usepackage{hyperref}\n");
    latex.push_str("\\usepackage{xcolor}\n\n");

    if !document.metadata.title.is_empty() {
        writeln!(
            latex,
            "\\title{{{}}}",
            escape_latex(&document.metadata.title)
        )
        .expect("writing to a String cannot fail");
    }
    if !document.metadata.author.is_empty() {
        writeln!(
            latex,
            "\\author{{{}}}",
            escape_latex(&document.metadata.author)
        )
        .expect("writing to a String cannot fail");
    }
    latex.push_str("\\date{}\n");
    latex.push_str("\\begin{document}\n");

    if !document.metadata.title.is_empty() || !document.metadata.author.is_empty() {
        latex.push_str("\\maketitle\n\n");
        if has_running_text {
            latex.push_str("\\thispagestyle{ingecalc}\n\n");
        }
    }

    for (page_index, page) in document.pages.iter().enumerate() {
        if page_index > 0 {
            latex.push_str("\\newpage\n\n");
        }

        for region in &page.regions {
            match region {
                Region::Math(math) => {
                    let source = if math.source.trim().is_empty() {
                        &math.ascii
                    } else {
                        &math.source
                    };
                    let trailing_equals = source.trim_end().ends_with('=');
                    latex.push_str("\\begin{equation*}\n");
                    latex.push_str(source.trim());
                    if let Some(result) = math_results.get(&math.id) {
                        let result = escape_latex(result);
                        if trailing_equals {
                            latex.push_str(&format!("\\quad \\textbf{{\\text{{{result}}}}}"));
                        } else {
                            latex.push_str(&format!(" = \\textbf{{\\text{{{result}}}}}"));
                        }
                    }
                    latex.push_str("\n\\end{equation*}\n\n");
                }
                Region::Text(text) => {
                    if text
                        .role
                        .as_deref()
                        .is_some_and(|role| role == "header" || role == "footer")
                    {
                        continue;
                    }
                    latex.push_str(&blocks_to_latex(&parse_html(&text.html)));
                    latex.push('\n');
                }
                Region::Image(image) => {
                    if let Some((extension, bytes)) = read_image(&image.src)? {
                        fs::create_dir_all(&assets_dir)?;
                        image_index += 1;
                        let file_name = format!("image_{image_index}.{extension}");
                        fs::write(assets_dir.join(&file_name), bytes)?;
                        writeln!(
                            latex,
                            "\\begin{{figure}}[h]\n\\centering\n\\includegraphics[width=0.8\\linewidth]{{assets/{file_name}}}\n\\end{{figure}}\n"
                        )
                        .expect("writing to a String cannot fail");
                    } else {
                        writeln!(latex, "% Imagen omitida: {}", escape_latex(&image.src))
                            .expect("writing to a String cannot fail");
                    }
                }
                Region::Plot(plot) => {
                    let expressions = plot
                        .series
                        .iter()
                        .filter(|series| series.visible)
                        .map(|series| escape_latex(&series.expression))
                        .collect::<Vec<_>>()
                        .join(", ");
                    writeln!(
                        latex,
                        "\\begin{{center}}\\fbox{{\\parbox{{0.8\\linewidth}}{{Gráfico 2D: {expressions}}}}}\\end{{center}}\n"
                    )
                    .expect("writing to a String cannot fail");
                }
            }
        }
    }

    latex.push_str("\\end{document}\n");
    fs::write(output, latex)?;
    Ok(())
}

fn blocks_to_latex(blocks: &[Block]) -> String {
    let mut latex = String::new();
    let mut in_list = false;

    for block in blocks {
        if !matches!(block, Block::ListItem(_)) && in_list {
            latex.push_str("\\end{itemize}\n\n");
            in_list = false;
        }

        match block {
            Block::Paragraph(runs) => {
                latex.push_str(&runs_to_latex(runs));
                latex.push_str("\n\n");
            }
            Block::Heading(level, runs) => {
                let command = match *level {
                    1 => "section",
                    2 => "subsection",
                    3 => "subsubsection",
                    _ => "paragraph",
                };
                writeln!(latex, "\\{command}*{{{}}}", runs_to_latex(runs))
                    .expect("writing to a String cannot fail");
                latex.push('\n');
            }
            Block::ListItem(runs) => {
                if !in_list {
                    latex.push_str("\\begin{itemize}\n");
                    in_list = true;
                }
                latex.push_str("\\item ");
                latex.push_str(&runs_to_latex(runs));
                latex.push('\n');
            }
            Block::Table(table) => {
                let columns = table
                    .rows
                    .iter()
                    .map(|row| row.cells.len())
                    .max()
                    .unwrap_or(0);

                if columns > 0 {
                    let alignment = format!(
                        "|{}|",
                        std::iter::repeat_n("l", columns)
                            .collect::<Vec<_>>()
                            .join("|")
                    );
                    writeln!(
                        latex,
                        "\\begin{{center}}\n\\begin{{tabular}}{{{alignment}}}\n\\hline"
                    )
                    .expect("writing to a String cannot fail");

                    for row in &table.rows {
                        for column in 0..columns {
                            if column > 0 {
                                latex.push_str(" & ");
                            }
                            if let Some(cell) = row.cells.get(column) {
                                let content = runs_to_latex(&cell.runs);
                                if cell.header {
                                    latex.push_str("\\textbf{");
                                    latex.push_str(&content);
                                    latex.push('}');
                                } else {
                                    latex.push_str(&content);
                                }
                            }
                        }
                        latex.push_str(" \\\\\n\\hline\n");
                    }

                    latex.push_str("\\end{tabular}\n\\end{center}\n\n");
                }
            }
        }
    }

    if in_list {
        latex.push_str("\\end{itemize}\n");
    }

    latex
}

fn runs_to_latex(runs: &[Run]) -> String {
    runs.iter().map(run_to_latex).collect::<Vec<_>>().join("")
}

fn run_to_latex(run: &Run) -> String {
    let mut text = if let Some(math) = &run.math {
        format!("${}$", math.latex)
    } else {
        escape_latex(&run.text).replace('\n', "\\\\\n")
    };

    if run.code {
        text = format!("\\texttt{{{text}}}");
    } else if let Some(font_family) = &run.font_family {
        let family = font_family.to_lowercase();
        text = if family.contains("courier")
            || family.contains("mono")
            || family.contains("consolas")
        {
            format!("\\texttt{{{text}}}")
        } else if family.contains("times") || family.contains("georgia") || family.contains("serif")
        {
            format!("\\textrm{{{text}}}")
        } else {
            format!("\\textsf{{{text}}}")
        };
    }
    if run.bold {
        text = format!("\\textbf{{{text}}}");
    }
    if run.italic {
        text = format!("\\emph{{{text}}}");
    }
    if run.underline {
        text = format!("\\underline{{{text}}}");
    }
    if let Some(link) = &run.link {
        text = format!("\\href{{{}}}{{{text}}}", escape_latex(link));
    }
    if let Some(font_size) = run.font_size {
        let points = font_size * 0.75;
        text = format!(
            "{{\\fontsize{{{points:.1}pt}}{{{:.1}pt}}\\selectfont {text}}}",
            points * 1.2
        );
    }
    if let Some(color) = run.color.as_deref().and_then(css_color_rgb) {
        text = format!("\\textcolor[rgb]{{{color}}}{{{text}}}");
    }

    text
}

fn css_color_rgb(color: &str) -> Option<String> {
    let hex = color.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    let channels: Vec<u8> = (0..3)
        .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;

    Some(format!(
        "{:.3},{:.3},{:.3}",
        f64::from(channels[0]) / 255.0,
        f64::from(channels[1]) / 255.0,
        f64::from(channels[2]) / 255.0
    ))
}

fn px_to_cm(value: f64) -> String {
    let centimeters = value.max(0.0) / 96.0 * 2.54;
    format!("{:.2}", (centimeters * 100.0).round() / 100.0)
}

fn running_text_to_latex(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;

    loop {
        let lower = rest.to_ascii_lowercase();
        if let Some(index) = lower.find("{page}") {
            result.push_str(&escape_latex(&rest[..index]));
            result.push_str("\\thepage{}");
            rest = &rest[index + "{page}".len()..];
        } else {
            result.push_str(&escape_latex(rest));
            break;
        }
    }

    result
}

fn escape_latex(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        match character {
            '\\' => escaped.push_str("\\textbackslash{}"),
            '{' => escaped.push_str("\\{"),
            '}' => escaped.push_str("\\}"),
            '$' => escaped.push_str("\\$"),
            '&' => escaped.push_str("\\&"),
            '#' => escaped.push_str("\\#"),
            '_' => escaped.push_str("\\_"),
            '%' => escaped.push_str("\\%"),
            '~' => escaped.push_str("\\textasciitilde{}"),
            '^' => escaped.push_str("\\textasciicircum{}"),
            _ => escaped.push(character),
        }
    }

    escaped
}

fn read_image(source: &str) -> Result<Option<(String, Vec<u8>)>, ExportError> {
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
        if !latex_supported_image(extension) {
            return Ok(None);
        }
        let bytes = base64::engine::general_purpose::STANDARD.decode(payload)?;
        return Ok(Some((extension.to_string(), bytes)));
    }

    let path = PathBuf::from(source);
    if path.exists() {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("bin");
        if !latex_supported_image(extension) {
            return Ok(None);
        }
        return Ok(Some((extension.to_string(), fs::read(path)?)));
    }

    Ok(None)
}

fn latex_supported_image(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "pdf"
    )
}

pub(crate) fn mime_to_extension(mime: &str) -> Option<&'static str> {
    match mime {
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/gif" => Some("gif"),
        "image/svg+xml" => Some("svg"),
        "image/webp" => Some("webp"),
        "image/bmp" => Some("bmp"),
        _ => None,
    }
}

pub(crate) fn extension_to_mime(extension: &str) -> &'static str {
    match extension.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
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

    #[test]
    fn exports_latex_document() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join(format!("latex-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("document.tex");

        export_latex(&Document::default(), &path).expect("LaTeX export");

        let text = fs::read_to_string(&path).expect("read exported LaTeX");
        assert!(text.contains("\\begin{document}"));
        assert!(text.contains("\\begin{equation*}"));
        assert!(text.contains("2+2"));
        assert!(text.contains("\\textbf{\\text{4}}"));
        assert!(text.contains("\\usepackage{amsmath}"));
        assert!(text.contains("\\ifPDFTeX"));

        let pdftex_branch = text.split("\\else").next().expect("PDFTeX branch");
        assert!(pdftex_branch.contains("\\usepackage{amssymb}"));

        let unicode_branch = text.split("\\else").nth(1).expect("Unicode branch");
        assert!(unicode_branch.contains("\\usepackage{unicode-math}"));
        assert!(!unicode_branch.contains("\\usepackage{amssymb}"));

        fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn reads_running_text_from_header_and_footer_regions() {
        let mut document = Document::default();
        document.pages[0].regions.push(Region::Text(TextRegion {
            id: "header".into(),
            x: 48.0,
            y: 0.0,
            w: 754.0,
            h: 48.0,
            html: "<p>Informe {page}</p>".into(),
            role: Some("header".into()),
            pinned: false,
        }));
        document.pages[0].regions.push(Region::Text(TextRegion {
            id: "footer".into(),
            x: 48.0,
            y: 1052.0,
            w: 754.0,
            h: 48.0,
            html: "<p>Página {page}</p>".into(),
            role: Some("footer".into()),
            pinned: false,
        }));

        assert_eq!(
            running_text_from_regions(&document, "header"),
            Some("Informe {page}".into())
        );
        assert_eq!(
            running_text_from_regions(&document, "footer"),
            Some("Página {page}".into())
        );
    }

    #[test]
    fn exports_table_with_header() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("latex-table");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("document.tex");

        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Text(TextRegion {
            id: "text-table".into(),
            x: 60.0,
            y: 60.0,
            w: 500.0,
            h: 160.0,
            html: r#"<table><tbody><tr><th><p>Nombre</p></th><th><p>Valor</p></th></tr><tr><td><p>Uno</p></td><td><p><strong>2</strong></p></td></tr></tbody></table>"#.into(),
            role: None,
            pinned: false,
        })];

        export_latex(&document, &path).expect("LaTeX export");
        let text = fs::read_to_string(&path).expect("read exported LaTeX");
        assert!(text.contains("\\begin{tabular}{|l|l|}"));
        assert!(text.contains("\\textbf{Nombre}"));
        assert!(text.contains("Uno & \\textbf{2}"));

        fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn exports_text_color_and_font_size() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("latex-color-size");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("document.tex");

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

        export_latex(&document, &path).expect("LaTeX export");
        let text = fs::read_to_string(&path).expect("read exported LaTeX");
        assert!(text.contains("\\textcolor[rgb]{1.000,0.000,0.000}"));
        assert!(text.contains("\\fontsize{15.0pt}{18.0pt}"));
        assert!(text.contains("\\textsf{"));

        fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn exports_page_geometry_and_running_text() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("latex-page-setup");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("document.tex");

        let mut document = Document::default();
        document.settings.page.header = "Informe {page}".into();
        document.settings.page.footer = "Página {page}".into();
        document.settings.page.margin_top = 96.0;
        document.settings.page.margin_right = 48.0;
        document.settings.page.margin_bottom = 72.0;
        document.settings.page.margin_left = 60.0;

        export_latex(&document, &path).expect("LaTeX export");
        let text = fs::read_to_string(&path).expect("read exported LaTeX");

        assert!(text.contains("top=2.54cm"));
        assert!(text.contains("right=1.27cm"));
        assert!(text.contains("bottom=1.91cm"));
        assert!(text.contains("left=1.59cm"));
        assert!(text.contains("\\newcommand{\\ps@ingecalc}"));
        assert!(text.contains("\\@oddhead"));
        assert!(text.contains("Informe \\thepage{}"));
        assert!(text.contains("\\@oddfoot"));
        assert!(text.contains("Página \\thepage{}"));

        fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn exports_inline_math_from_text_regions() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("latex-inline");
        fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("document.tex");

        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Text(TextRegion {
            id: "text-1".into(),
            x: 60.0,
            y: 60.0,
            w: 500.0,
            h: 100.0,
            html: r#"<p>Área: <span data-latex="x^2" data-ascii="x^2" data-inline-math=""></span> unidades</p>"#.into(),
            role: None,
            pinned: false,
        })];

        export_latex(&document, &path).expect("LaTeX export");
        let text = fs::read_to_string(&path).expect("read exported LaTeX");
        assert!(text.contains("$x^2$"));
        assert!(text.contains("unidades"));

        fs::remove_dir_all(directory).ok();
    }
}
