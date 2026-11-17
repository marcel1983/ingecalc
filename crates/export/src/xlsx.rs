// SPDX-License-Identifier: GPL-3.0-or-later

//! Excel workbook export with a formatted document sheet and a live-formula
//! calculation sheet.
//!
//! The `.xlsx` file is written directly as an Open XML package. Formulas are
//! translated from IngeCalc's ASCII math into Excel syntax whenever the
//! expression is scalar and does not depend on explicit unit conversions.
//! Expressions with units keep their evaluated value and unit label, so the
//! workbook remains numerically consistent with the document.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::io::{Cursor, Write as _};

use doc_core::{Document, Region};
use math_ast::{BinaryOp, Expr, UnaryOp};
use math_eval::{NumberFormat, UnitPreferences};
use math_units::UnitRegistry;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::html::{parse_html, Block, Run};
use crate::ExportError;

const SHEET_DOCUMENT: &str = "Documento";
const SHEET_CALCULATIONS: &str = "Cálculos";

const STYLE_TITLE: u32 = 1;
const STYLE_SUBTITLE: u32 = 2;
const STYLE_SECTION: u32 = 3;
const STYLE_FORMULA: u32 = 5;
const STYLE_NOTE: u32 = 7;
const STYLE_HEADER: u32 = 8;
const STYLE_NUMBER: u32 = 9;
const STYLE_BODY: u32 = 10;
const STYLE_UNIT: u32 = 11;
const STYLE_HEADING: u32 = 12;
const STYLE_CODE: u32 = 13;
const STYLE_RESULT_TEXT: u32 = 14;
const STYLE_CENTER: u32 = 15;
const STYLE_WARNING: u32 = 16;
const STYLE_RESULT_NUMBER: u32 = 17;

pub fn export_xlsx(document: &Document) -> Result<Vec<u8>, ExportError> {
    let evaluations = evaluate_regions(document);
    let (calculations_sheet, calculation_rows) = build_calculations_sheet(document, &evaluations);
    let document_sheet = build_document_sheet(document, &evaluations, &calculation_rows);

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    write_file(
        &mut zip,
        "[Content_Types].xml",
        content_types(document_sheet.rows.len(), calculations_sheet.rows.len()),
        options,
    )?;
    write_file(&mut zip, "_rels/.rels", root_relationships(), options)?;
    write_file(
        &mut zip,
        "docProps/core.xml",
        core_properties(document),
        options,
    )?;
    write_file(&mut zip, "docProps/app.xml", app_properties(), options)?;
    write_file(&mut zip, "xl/workbook.xml", workbook_xml(), options)?;
    write_file(
        &mut zip,
        "xl/_rels/workbook.xml.rels",
        workbook_relationships(),
        options,
    )?;
    write_file(&mut zip, "xl/styles.xml", styles_xml(), options)?;
    write_file(
        &mut zip,
        "xl/worksheets/sheet1.xml",
        document_sheet.to_xml(),
        options,
    )?;
    write_file(
        &mut zip,
        "xl/worksheets/sheet2.xml",
        calculations_sheet.to_xml(),
        options,
    )?;

    Ok(zip.finish()?.into_inner())
}

fn write_file(
    zip: &mut ZipWriter<Cursor<Vec<u8>>>,
    name: &str,
    contents: impl AsRef<[u8]>,
    options: SimpleFileOptions,
) -> Result<(), ExportError> {
    zip.start_file(name, options)?;
    zip.write_all(contents.as_ref())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Workbook model
// ---------------------------------------------------------------------------

struct Sheet {
    rows: BTreeMap<u32, Row>,
    next_row: u32,
    merges: Vec<String>,
    row_breaks: Vec<u32>,
    freeze: Option<u32>,
    columns: Vec<Column>,
    landscape: bool,
}

struct Row {
    index: u32,
    height: Option<f64>,
    cells: BTreeMap<u32, Cell>,
}

struct Column {
    min: u32,
    max: u32,
    width: f64,
}

struct Cell {
    column: u32,
    style: u32,
    value: CellValue,
}

enum CellValue {
    Text(String),
    Number(f64),
    Formula(String, f64),
}

impl Sheet {
    fn new(name: &str, columns: Vec<Column>, freeze: Option<u32>) -> Self {
        let _ = name;
        Self {
            rows: BTreeMap::new(),
            next_row: 1,
            merges: Vec::new(),
            row_breaks: Vec::new(),
            freeze,
            columns,
            landscape: false,
        }
    }

    fn landscape(mut self) -> Self {
        self.landscape = true;
        self
    }

    fn add_row(&mut self) -> u32 {
        let index = self.next_row;
        self.next_row += 1;
        self.rows.insert(
            index,
            Row {
                index,
                height: None,
                cells: BTreeMap::new(),
            },
        );
        index
    }

    fn set_height(&mut self, row: u32, height: f64) {
        if let Some(entry) = self.rows.get_mut(&row) {
            entry.height = Some(height.max(15.0));
        }
    }

    fn set_cell(&mut self, row: u32, column: u32, value: CellValue, style: u32) {
        if let Some(entry) = self.rows.get_mut(&row) {
            entry.cells.insert(
                column,
                Cell {
                    column,
                    style,
                    value,
                },
            );
        }
    }

    fn text(&mut self, row: u32, column: u32, text: &str, style: u32) {
        if text.trim().is_empty() {
            return;
        }
        self.set_cell(row, column, CellValue::Text(text.to_string()), style);
    }

    fn number(&mut self, row: u32, column: u32, value: f64, style: u32) {
        if !value.is_finite() {
            return;
        }
        self.set_cell(row, column, CellValue::Number(value), style);
    }

    fn formula(&mut self, row: u32, column: u32, formula: &str, cached: f64, style: u32) {
        if !cached.is_finite() {
            return;
        }
        self.set_cell(
            row,
            column,
            CellValue::Formula(formula.to_string(), cached),
            style,
        );
    }

    fn merge(&mut self, row: u32, first: u32, last: u32) {
        if first >= last {
            return;
        }
        self.merges.push(format!(
            "{}{}:{}{}",
            column_name(first),
            row,
            column_name(last),
            row
        ));
    }

    fn page_break(&mut self, row: u32) {
        self.row_breaks.push(row);
    }

    fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>");
        xml.push_str(
            "<worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
             xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">",
        );

        xml.push_str("<sheetPr><pageSetUpPr fitToPage=\"1\"/></sheetPr>");

        if let Some(freeze) = self.freeze {
            write!(
                xml,
                "<sheetViews><sheetView workbookViewId=\"0\"><pane ySplit=\"{freeze}\" \
                 topLeftCell=\"A{}\" activePane=\"bottomLeft\" state=\"frozen\"/></sheetView></sheetViews>",
                freeze + 1
            )
            .expect("writing to a String cannot fail");
        }

        xml.push_str("<sheetFormatPr defaultRowHeight=\"15\"/>");

        if !self.columns.is_empty() {
            xml.push_str("<cols>");
            for column in &self.columns {
                write!(
                    xml,
                    "<col min=\"{}\" max=\"{}\" width=\"{:.2}\" customWidth=\"1\"/>",
                    column.min, column.max, column.width
                )
                .expect("writing to a String cannot fail");
            }
            xml.push_str("</cols>");
        }

        xml.push_str("<sheetData>");
        for row in self.rows.values() {
            xml.push('<');
            write!(xml, "row r=\"{}\"", row.index).expect("writing to a String cannot fail");
            if let Some(height) = row.height {
                write!(xml, " ht=\"{height:.2}\" customHeight=\"1\"")
                    .expect("writing to a String cannot fail");
            }
            xml.push('>');
            for cell in row.cells.values() {
                write_cell(&mut xml, row.index, cell);
            }
            xml.push_str("</row>");
        }
        xml.push_str("</sheetData>");

        if !self.merges.is_empty() {
            write!(xml, "<mergeCells count=\"{}\">", self.merges.len())
                .expect("writing to a String cannot fail");
            for reference in &self.merges {
                write!(xml, "<mergeCell ref=\"{}\"/>", xml_escape(reference))
                    .expect("writing to a String cannot fail");
            }
            xml.push_str("</mergeCells>");
        }

        if !self.row_breaks.is_empty() {
            write!(
                xml,
                "<rowBreaks count=\"{}\" manualBreakCount=\"{}\">",
                self.row_breaks.len(),
                self.row_breaks.len()
            )
            .expect("writing to a String cannot fail");
            for row in &self.row_breaks {
                write!(xml, "<brk id=\"{row}\" max=\"16383\" man=\"1\"/>")
                    .expect("writing to a String cannot fail");
            }
            xml.push_str("</rowBreaks>");
        }

        xml.push_str(
            "<pageMargins left=\"0.4\" right=\"0.4\" top=\"0.5\" bottom=\"0.5\" header=\"0.2\" footer=\"0.2\"/>",
        );
        write!(
            xml,
            "<pageSetup orientation=\"{}\" fitToWidth=\"1\" fitToHeight=\"0\"/>",
            if self.landscape {
                "landscape"
            } else {
                "portrait"
            }
        )
        .expect("writing to a String cannot fail");
        xml.push_str("</worksheet>");
        xml
    }
}

fn write_cell(xml: &mut String, row: u32, cell: &Cell) {
    let reference = format!("{}{}", column_name(cell.column), row);
    match &cell.value {
        CellValue::Text(value) => {
            write!(
                xml,
                "<c r=\"{}\" s=\"{}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                xml_escape(&reference),
                cell.style,
                xml_escape(value)
            )
            .expect("writing to a String cannot fail");
        }
        CellValue::Number(value) => {
            write!(
                xml,
                "<c r=\"{}\" s=\"{}\"><v>{}</v></c>",
                xml_escape(&reference),
                cell.style,
                number_text(*value)
            )
            .expect("writing to a String cannot fail");
        }
        CellValue::Formula(formula, cached) => {
            write!(
                xml,
                "<c r=\"{}\" s=\"{}\"><f>{}</f><v>{}</v></c>",
                xml_escape(&reference),
                cell.style,
                xml_escape(formula),
                number_text(*cached)
            )
            .expect("writing to a String cannot fail");
        }
    }
}

fn column_name(mut column: u32) -> String {
    let mut name = String::new();
    while column > 0 {
        let remainder = (column - 1) % 26;
        name.insert(0, char::from(b'A' + remainder as u8));
        column = (column - 1) / 26;
    }
    name
}

fn number_text(value: f64) -> String {
    if !value.is_finite() {
        return "0".into();
    }
    let mut text = format!("{value:.15}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" {
        "0".into()
    } else {
        text
    }
}

// ---------------------------------------------------------------------------
// Document sheet
// ---------------------------------------------------------------------------

fn build_document_sheet(
    document: &Document,
    evaluations: &HashMap<String, EvalInfo>,
    calculation_rows: &HashMap<String, u32>,
) -> Sheet {
    let mut sheet = Sheet::new(
        SHEET_DOCUMENT,
        vec![
            Column {
                min: 1,
                max: 1,
                width: 3.0,
            },
            Column {
                min: 2,
                max: 2,
                width: 72.0,
            },
            Column {
                min: 3,
                max: 3,
                width: 28.0,
            },
            Column {
                min: 4,
                max: 4,
                width: 14.0,
            },
            Column {
                min: 5,
                max: 8,
                width: 14.0,
            },
        ],
        None,
    );

    let title_row = sheet.add_row();
    sheet.text(title_row, 2, document.metadata.title.trim(), STYLE_TITLE);
    sheet.merge(title_row, 2, 4);
    sheet.set_height(title_row, 30.0);

    let subtitle_row = sheet.add_row();
    let subtitle = if document.metadata.author.trim().is_empty() {
        "Generado por IngeCalc".to_string()
    } else {
        format!(
            "{} · Generado por IngeCalc",
            document.metadata.author.trim()
        )
    };
    sheet.text(subtitle_row, 2, &subtitle, STYLE_SUBTITLE);
    sheet.merge(subtitle_row, 2, 4);

    let note_row = sheet.add_row();
    sheet.text(
        note_row,
        2,
        "Vista formateada del documento. Las fórmulas editables están en la hoja «Cálculos».",
        STYLE_NOTE,
    );
    sheet.merge(note_row, 2, 4);
    sheet.set_height(note_row, 26.0);

    for (page_index, page) in document.pages.iter().enumerate() {
        let section = sheet.add_row();
        sheet.text(
            section,
            2,
            &format!("Página {}", page_index + 1),
            STYLE_SECTION,
        );
        sheet.merge(section, 2, 4);
        sheet.set_height(section, 22.0);

        let mut regions = page.regions.iter().collect::<Vec<_>>();
        regions.sort_by(|left, right| {
            let (lx, ly) = region_position(left);
            let (rx, ry) = region_position(right);
            ly.partial_cmp(&ry)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(lx.partial_cmp(&rx).unwrap_or(std::cmp::Ordering::Equal))
        });

        for region in regions {
            match region {
                Region::Text(text) => {
                    for block in parse_html(&text.html) {
                        write_text_block(&mut sheet, block);
                    }
                }
                Region::Math(math) => {
                    let row = sheet.add_row();
                    let pretty = pretty_formula(&math.ascii);
                    sheet.text(row, 2, &pretty, STYLE_FORMULA);
                    sheet.set_height(row, estimated_height(&pretty, 62.0, 32.0));

                    if let Some(info) = evaluations.get(&math.id) {
                        if let Some(error) = &info.error {
                            sheet.text(row, 3, "ERROR", STYLE_WARNING);
                            sheet.text(row, 4, &truncate(error, 80), STYLE_NOTE);
                        } else {
                            let linked = calculation_rows.get(&math.id).zip(info.number);
                            if let Some((formula_row, value)) = linked {
                                sheet.formula(
                                    row,
                                    3,
                                    &format!("'{SHEET_CALCULATIONS}'!$D${formula_row}"),
                                    value,
                                    STYLE_RESULT_NUMBER,
                                );
                            } else {
                                sheet.text(row, 3, &info.value_text, STYLE_RESULT_TEXT);
                            }
                            sheet.text(row, 4, &info.unit, STYLE_UNIT);
                        }
                    }
                }
                Region::Image(_) => {
                    let row = sheet.add_row();
                    sheet.text(
                        row,
                        2,
                        "[Imagen no incluida en el libro de Excel]",
                        STYLE_NOTE,
                    );
                    sheet.merge(row, 2, 4);
                }
                Region::Plot(plot) => {
                    let row = sheet.add_row();
                    let expressions = plot
                        .series
                        .iter()
                        .filter(|series| series.visible)
                        .map(|series| series.expression.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    sheet.text(row, 2, &format!("[Gráfico 2D: {expressions}]"), STYLE_NOTE);
                    sheet.merge(row, 2, 4);
                }
            }
        }

        sheet.add_row();
        if page_index + 1 < document.pages.len() {
            sheet.page_break(sheet.next_row.saturating_sub(1));
        }
    }

    sheet
}

fn write_text_block(sheet: &mut Sheet, block: Block) {
    match block {
        Block::Heading(level, runs) => {
            let row = sheet.add_row();
            let text = runs_text(&runs);
            sheet.text(row, 2, &text, STYLE_HEADING);
            sheet.merge(row, 2, 4);
            sheet.set_height(row, if level <= 2 { 26.0 } else { 22.0 });
        }
        Block::Paragraph(runs) => {
            write_paragraph(sheet, runs, STYLE_BODY);
        }
        Block::ListItem(runs) => {
            let text = format!("• {}", runs_text(&runs));
            write_paragraph(
                sheet,
                vec![Run {
                    text,
                    ..Run::default()
                }],
                STYLE_BODY,
            );
        }
        Block::Table(table) => {
            for row in &table.rows {
                let sheet_row = sheet.add_row();
                for (index, cell) in row.cells.iter().enumerate() {
                    let column = 2 + index as u32;
                    if column > 8 {
                        break;
                    }
                    let text = runs_text(&cell.runs);
                    sheet.text(
                        sheet_row,
                        column,
                        &text,
                        if cell.header {
                            STYLE_HEADER
                        } else {
                            STYLE_BODY
                        },
                    );
                }
                let height = row
                    .cells
                    .iter()
                    .map(|cell| estimated_height(&runs_text(&cell.runs), 28.0, 18.0))
                    .fold(18.0_f64, f64::max);
                sheet.set_height(sheet_row, height);
            }
            sheet.add_row();
        }
    }
}

fn write_paragraph(sheet: &mut Sheet, runs: Vec<Run>, style: u32) {
    let row = sheet.add_row();
    let text = runs_text(&runs);
    sheet.text(row, 2, &text, style);
    sheet.merge(row, 2, 4);
    sheet.set_height(row, estimated_height(&text, 72.0, 18.0));
}

fn runs_text(runs: &[Run]) -> String {
    let mut text = String::new();
    for run in runs {
        if let Some(math) = &run.math {
            if !text.is_empty() && !text.ends_with(' ') {
                text.push(' ');
            }
            text.push_str(&pretty_formula(&math.ascii));
            text.push(' ');
        } else {
            text.push_str(&run.text);
        }
    }
    text.trim().to_string()
}

fn region_position(region: &Region) -> (f64, f64) {
    match region {
        Region::Math(math) => (math.x, math.y),
        Region::Text(text) => (text.x, text.y),
        Region::Image(image) => (image.x, image.y),
        Region::Plot(plot) => (plot.x, plot.y),
    }
}

// ---------------------------------------------------------------------------
// Calculations sheet
// ---------------------------------------------------------------------------

struct EvalInfo {
    value_text: String,
    number: Option<f64>,
    unit: String,
    error: Option<String>,
}

fn evaluate_regions(document: &Document) -> HashMap<String, EvalInfo> {
    let mut items = Vec::new();
    for page in &document.pages {
        for region in &page.regions {
            if let Region::Math(math) = region {
                if !math.ascii.trim().is_empty() {
                    items.push((math.id.clone(), math.ascii.clone()));
                }
            }
        }
    }

    let constants = document
        .settings
        .custom_constants
        .iter()
        .map(|constant| (constant.name.clone(), constant.expression.clone()))
        .collect::<Vec<_>>();

    let preferences = UnitPreferences::new(
        document.settings.units.clone(),
        document.settings.unit_overrides.clone(),
    );
    let format = NumberFormat::from_settings(
        &document.settings.notation,
        document.settings.precision,
        document.settings.trim_zeros,
        &document.settings.fractions,
    );

    let mut evaluations = HashMap::new();
    for evaluation in math_eval::evaluate_document_with_constants_and_preferences(
        &items,
        &constants,
        &preferences,
    ) {
        let info = match evaluation.result {
            Ok(mut value) => {
                value.convert_units(&preferences);
                let display = value.display_with_format(&format);
                let unit = value
                    .as_quantity()
                    .map(|quantity| quantity.unit.symbol.clone())
                    .unwrap_or_default();
                let number = value
                    .as_quantity()
                    .map(|quantity| quantity.display_value())
                    .or_else(|| value.scalar());
                let suffix = if unit.is_empty() {
                    String::new()
                } else if unit.starts_with('°') || unit.starts_with('%') {
                    unit.clone()
                } else {
                    format!(" {unit}")
                };
                let value_text = display
                    .strip_suffix(&suffix)
                    .unwrap_or(&display)
                    .to_string();
                EvalInfo {
                    value_text,
                    number,
                    unit,
                    error: None,
                }
            }
            Err(error) => EvalInfo {
                value_text: "ERROR".into(),
                number: None,
                unit: String::new(),
                error: Some(error.to_string()),
            },
        };
        evaluations.insert(evaluation.id, info);
    }
    evaluations
}

fn build_calculations_sheet(
    document: &Document,
    evaluations: &HashMap<String, EvalInfo>,
) -> (Sheet, HashMap<String, u32>) {
    let mut sheet = Sheet::new(
        SHEET_CALCULATIONS,
        vec![
            Column {
                min: 1,
                max: 1,
                width: 4.0,
            },
            Column {
                min: 2,
                max: 2,
                width: 62.0,
            },
            Column {
                min: 3,
                max: 3,
                width: 62.0,
            },
            Column {
                min: 4,
                max: 4,
                width: 22.0,
            },
            Column {
                min: 5,
                max: 5,
                width: 14.0,
            },
            Column {
                min: 6,
                max: 6,
                width: 40.0,
            },
        ],
        Some(3),
    )
    .landscape();

    let title = sheet.add_row();
    sheet.text(title, 1, "Cálculos", STYLE_TITLE);
    sheet.merge(title, 1, 6);
    sheet.set_height(title, 30.0);

    let note = sheet.add_row();
    sheet.text(
        note,
        1,
        "Las fórmulas de la columna D son reales: puede editar los valores y Excel recalculará. \
         La columna C muestra la fórmula equivalente. Las expresiones con unidades conservan el \
         valor calculado y su unidad.",
        STYLE_NOTE,
    );
    sheet.merge(note, 1, 6);
    sheet.set_height(note, 42.0);

    let header = sheet.add_row();
    for (column, label) in [
        (1, "#"),
        (2, "Fórmula del documento"),
        (3, "Fórmula Excel"),
        (4, "Resultado"),
        (5, "Unidad"),
        (6, "Nota"),
    ] {
        sheet.text(header, column, label, STYLE_HEADER);
    }
    sheet.set_height(header, 22.0);

    let math_regions = collect_math_regions(document);
    let mut parsed: Vec<Option<Expr>> = Vec::with_capacity(math_regions.len());
    let mut names: HashMap<String, usize> = HashMap::new();
    for (index, math) in math_regions.iter().enumerate() {
        let expression = math_parse::parse(&math.ascii).ok();
        if let Some(Expr::Assign { name, .. }) = &expression {
            names.insert(name.clone(), index);
        }
        parsed.push(expression);
    }

    let defined = names.keys().cloned().collect::<HashSet<_>>();
    let registry = UnitRegistry::global();
    let mut rows = Vec::with_capacity(math_regions.len());
    let mut calculation_rows = HashMap::new();
    for (index, math) in math_regions.iter().enumerate() {
        let row = sheet.add_row();
        rows.push(row);
        calculation_rows.insert(math.id.clone(), row);
        sheet.number(row, 1, (index + 1) as f64, STYLE_CENTER);

        let pretty = pretty_formula(&math.ascii);
        sheet.text(row, 2, &pretty, STYLE_FORMULA);

        let evaluation = evaluations.get(&math.id);
        let definition_name = match &parsed[index] {
            Some(Expr::Assign { name, .. }) => Some(name.clone()),
            _ => None,
        };

        let formula = parsed[index].as_ref().and_then(|expression| {
            let value = match expression {
                Expr::Assign { value, .. } => value.as_ref(),
                other => other,
            };
            if contains_units(value, &defined, registry) {
                return None;
            }
            translate_expr(value, &names, &parsed, &defined, 0)
        });

        let number = evaluation.and_then(|info| info.number);
        let error = evaluation.and_then(|info| info.error.as_deref());

        if let (Some(formula), Some(value)) = (&formula, number) {
            if formula.len() <= 7000 {
                sheet.text(row, 3, formula, STYLE_CODE);
                sheet.formula(row, 4, formula, value, STYLE_NUMBER);
            } else {
                sheet.text(row, 3, "—", STYLE_NOTE);
                sheet.number(row, 4, value, STYLE_NUMBER);
            }
        } else {
            sheet.text(row, 3, "—", STYLE_NOTE);
            if let Some(value) = number {
                sheet.number(row, 4, value, STYLE_NUMBER);
            } else if let Some(info) = evaluation {
                sheet.text(row, 4, &truncate(&info.value_text, 120), STYLE_BODY);
            } else {
                sheet.text(row, 4, "—", STYLE_NOTE);
            }
        }

        if let Some(info) = evaluation {
            sheet.text(row, 5, &info.unit, STYLE_UNIT);
        }

        let note = if let Some(error) = error {
            format!("Error: {}", truncate(error, 140))
        } else if let Some(name) = definition_name {
            if formula.is_some() {
                format!("Definición de {name}")
            } else {
                format!("Constante de {name} (unidades o función no traducible)")
            }
        } else if formula.is_some() {
            "Expresión".to_string()
        } else if number.is_some() {
            "Valor calculado (unidades o función no traducible)".to_string()
        } else {
            "Sin resultado numérico".to_string()
        };
        sheet.text(row, 6, &note, STYLE_NOTE);

        let height = estimated_height(&pretty, 62.0, 30.0).max(estimated_height(
            formula.as_deref().unwrap_or("—"),
            62.0,
            30.0,
        ));
        sheet.set_height(row, height);
    }

    if math_regions.is_empty() {
        let row = sheet.add_row();
        sheet.text(
            row,
            1,
            "Este documento no contiene fórmulas matemáticas.",
            STYLE_NOTE,
        );
        sheet.merge(row, 1, 6);
    }

    (sheet, calculation_rows)
}

fn collect_math_regions(document: &Document) -> Vec<&doc_core::MathRegion> {
    let mut regions = Vec::new();
    for page in &document.pages {
        for region in &page.regions {
            if let Region::Math(math) = region {
                if !math.ascii.trim().is_empty() {
                    regions.push(math);
                }
            }
        }
    }
    regions
}

// ---------------------------------------------------------------------------
// Formula translation
// ---------------------------------------------------------------------------

fn translate_expr(
    expression: &Expr,
    names: &HashMap<String, usize>,
    parsed: &[Option<Expr>],
    defined: &HashSet<String>,
    depth: usize,
) -> Option<String> {
    if depth > 32 {
        return None;
    }

    match expression {
        Expr::Number(value) => Some(number_text(*value)),
        Expr::Symbol(name) => {
            if let Some(index) = names.get(name) {
                if parsed.get(*index).is_some() {
                    return Some(format!("'{SHEET_CALCULATIONS}'!$D${}", 4 + *index as u32));
                }
            }
            match name.as_str() {
                "pi" => Some("PI()".into()),
                "e" => Some("EXP(1)".into()),
                _ if defined.contains(name) => names
                    .get(name)
                    .map(|index| format!("'{SHEET_CALCULATIONS}'!$D${}", 4 + *index as u32)),
                _ => None,
            }
        }
        Expr::Unary { op, expr } => {
            let inner = translate_expr(expr, names, parsed, defined, depth + 1)?;
            Some(match op {
                UnaryOp::Neg => format!("-({inner})"),
                UnaryOp::Pos => format!("({inner})"),
            })
        }
        Expr::Binary { op, lhs, rhs } => {
            let left = translate_expr(lhs, names, parsed, defined, depth + 1)?;
            let right = translate_expr(rhs, names, parsed, defined, depth + 1)?;
            let operator = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Pow => "^",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                BinaryOp::Eq => "=",
                BinaryOp::Ne => "<>",
            };
            Some(format!("({left} {operator} {right})"))
        }
        Expr::Call { name, args } => translate_call(name, args, names, parsed, defined, depth),
        Expr::Assign { value, .. } => translate_expr(value, names, parsed, defined, depth + 1),
        Expr::FunctionDef { .. }
        | Expr::UnitDef { .. }
        | Expr::List(_)
        | Expr::Matrix(_)
        | Expr::Index { .. } => None,
    }
}

fn translate_call(
    name: &str,
    args: &[Expr],
    names: &HashMap<String, usize>,
    parsed: &[Option<Expr>],
    defined: &HashSet<String>,
    depth: usize,
) -> Option<String> {
    let translated = args
        .iter()
        .map(|argument| translate_expr(argument, names, parsed, defined, depth + 1))
        .collect::<Option<Vec<_>>>()?;

    let lower = name.to_ascii_lowercase();
    let call = |function: &str| Some(format!("{function}({})", translated.join(",")));

    match lower.as_str() {
        "sqrt" => call("SQRT"),
        "abs" => call("ABS"),
        "sin" => call("SIN"),
        "cos" => call("COS"),
        "tan" => call("TAN"),
        "asin" => call("ASIN"),
        "acos" => call("ACOS"),
        "atan" => call("ATAN"),
        "sinh" => call("SINH"),
        "cosh" => call("COSH"),
        "tanh" => call("TANH"),
        "asinh" => call("ASINH"),
        "acosh" => call("ACOSH"),
        "atanh" => call("ATANH"),
        "ln" => call("LN"),
        "log" => call("LOG10"),
        "exp" => call("EXP"),
        "sign" => call("SIGN"),
        "trunc" => call("TRUNC"),
        "frac" => translated.first().map(|value| format!("MOD({value},1)")),
        "floor" => translated.first().map(|value| format!("FLOOR({value},1)")),
        "ceil" => translated
            .first()
            .map(|value| format!("CEILING({value},1)")),
        "round" => translated.first().map(|value| format!("ROUND({value},0)")),
        "mod" => Some(format!(
            "MOD({},{})",
            translated.first()?,
            translated.get(1)?
        )),
        "gcd" => call("GCD"),
        "lcm" => call("LCM"),
        "factorial" => call("FACT"),
        "comb" => call("COMBIN"),
        "perm" => call("PERMUT"),
        "min" => call("MIN"),
        "max" => call("MAX"),
        "mean" => call("AVERAGE"),
        "if" => Some(format!(
            "IF({},{},{})",
            translated.first()?,
            translated.get(1)?,
            translated.get(2)?
        )),
        "atan2" => Some(format!(
            "ATAN2({},{})",
            translated.get(1)?,
            translated.first()?
        )),
        _ => None,
    }
}

fn contains_units(expression: &Expr, defined: &HashSet<String>, registry: &UnitRegistry) -> bool {
    match expression {
        Expr::Number(_) => false,
        Expr::Symbol(name) => !defined.contains(name) && registry.lookup(name).is_some(),
        Expr::Unary { expr, .. } => contains_units(expr, defined, registry),
        Expr::Binary { lhs, rhs, .. } => {
            contains_units(lhs, defined, registry) || contains_units(rhs, defined, registry)
        }
        Expr::Call { args, .. } => args
            .iter()
            .any(|argument| contains_units(argument, defined, registry)),
        Expr::Assign { value, .. } => contains_units(value, defined, registry),
        Expr::FunctionDef { body, .. } => contains_units(body, defined, registry),
        Expr::UnitDef { .. } => true,
        Expr::List(items) => items
            .iter()
            .any(|item| contains_units(item, defined, registry)),
        Expr::Matrix(rows) => rows
            .iter()
            .flatten()
            .any(|item| contains_units(item, defined, registry)),
        Expr::Index { target, indices } => {
            contains_units(target, defined, registry)
                || indices
                    .iter()
                    .any(|index| contains_units(index, defined, registry))
        }
    }
}

// ---------------------------------------------------------------------------
// Pretty formula display
// ---------------------------------------------------------------------------

fn pretty_formula(ascii: &str) -> String {
    let mut text = ascii.trim().to_string();
    text = text.replace("sqrt(", "√(");
    text = text.replace("*", " · ");
    text = text.replace("<=", "≤");
    text = text.replace(">=", "≥");
    text = text.replace("!=", "≠");
    text = text.replace("^2", "²");
    text = text.replace("^3", "³");
    for (word, symbol) in [
        ("epsilon", "ε"),
        ("alpha", "α"),
        ("beta", "β"),
        ("gamma", "γ"),
        ("delta", "δ"),
        ("theta", "θ"),
        ("lambda", "λ"),
        ("mu", "μ"),
        ("pi", "π"),
        ("rho", "ρ"),
        ("sigma", "σ"),
        ("tau", "τ"),
        ("phi", "φ"),
        ("psi", "ψ"),
        ("omega", "ω"),
    ] {
        text = replace_word(&text, word, symbol);
    }
    text
}

fn replace_word(source: &str, word: &str, replacement: &str) -> String {
    if word.is_empty() {
        return source.to_string();
    }
    let mut output = String::new();
    let mut rest = source;
    while let Some(position) = rest.find(word) {
        let before_ok = position == 0
            || rest[..position]
                .chars()
                .last()
                .is_none_or(|character| !is_identifier(character));
        let after = position + word.len();
        let after_ok = after >= rest.len()
            || rest[after..]
                .chars()
                .next()
                .is_none_or(|character| !character.is_alphanumeric());
        if before_ok && after_ok {
            output.push_str(&rest[..position]);
            output.push_str(replacement);
        } else {
            output.push_str(&rest[..after]);
        }
        rest = &rest[after..];
    }
    output.push_str(rest);
    output
}

fn is_identifier(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn estimated_height(text: &str, characters_per_line: f64, minimum: f64) -> f64 {
    let lines = (text.chars().count() as f64 / characters_per_line.max(1.0))
        .ceil()
        .max(1.0);
    (15.0 * lines + 5.0).max(minimum)
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut value = text.chars().take(limit).collect::<String>();
    value.push('…');
    value
}

fn xml_escape(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\t' | '\n' | '\r' => output.push(character),
            character if (character as u32) < 0x20 => {}
            character => output.push(character),
        }
    }
    output
}

// ---------------------------------------------------------------------------
// Package files
// ---------------------------------------------------------------------------

fn content_types(_document_rows: usize, _calculation_rows: usize) -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
<Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
</Types>"#
        .into()
}

fn root_relationships() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#
        .into()
}

fn workbook_relationships() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#
        .into()
}

fn workbook_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>
<sheet name="Documento" sheetId="1" r:id="rId1"/>
<sheet name="Cálculos" sheetId="2" r:id="rId2"/>
</sheets>
<calcPr calcId="191029" fullCalcOnLoad="1"/>
</workbook>"#
        .into()
}

fn core_properties(document: &Document) -> String {
    let title = xml_escape(document.metadata.title.trim());
    let author = xml_escape(document.metadata.author.trim());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:title>{title}</dc:title>
<dc:creator>{author}</dc:creator>
<cp:lastModifiedBy>IngeCalc</cp:lastModifiedBy>
</cp:coreProperties>"#
    )
}

fn app_properties() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
<Application>IngeCalc</Application>
</Properties>"#
        .into()
}

fn styles_xml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<numFmts count="2"><numFmt numFmtId="164" formatCode="0.############"/><numFmt numFmtId="165" formatCode="0.####"/></numFmts>
<fonts count="11">
<font><sz val="11"/><color rgb="FF1F2937"/><name val="Calibri"/></font>
<font><b/><sz val="18"/><color rgb="FF0F172A"/><name val="Calibri"/></font>
<font><i/><sz val="10"/><color rgb="FF64748B"/><name val="Calibri"/></font>
<font><b/><sz val="12"/><color rgb="FFFFFFFF"/><name val="Calibri"/></font>
<font><b/><sz val="11"/><color rgb="FF1F2937"/><name val="Calibri"/></font>
<font><sz val="12"/><color rgb="FF0F172A"/><name val="Cambria Math"/></font>
<font><b/><sz val="11"/><color rgb="FF047857"/><name val="Cambria Math"/></font>
<font><i/><sz val="10"/><color rgb="FF64748B"/><name val="Calibri"/></font>
<font><sz val="11"/><color rgb="FF1F2937"/><name val="Calibri"/></font>
<font><b/><sz val="14"/><color rgb="FF0F172A"/><name val="Calibri"/></font>
<font><sz val="10"/><color rgb="FF0F172A"/><name val="Consolas"/></font>
</fonts>
<fills count="8">
<fill><patternFill patternType="none"/></fill>
<fill><patternFill patternType="gray125"/></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FF0F172A"/><bgColor indexed="64"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFE2E8F0"/><bgColor indexed="64"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFECFDF5"/><bgColor indexed="64"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFF8FAFC"/><bgColor indexed="64"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFFEF3C7"/><bgColor indexed="64"/></patternFill></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FFF1F5F9"/><bgColor indexed="64"/></patternFill></fill>
</fills>
<borders count="4">
<border><left/><right/><top/><bottom/><diagonal/></border>
<border><left style="thin"><color rgb="FFCBD5E1"/></left><right style="thin"><color rgb="FFCBD5E1"/></right><top style="thin"><color rgb="FFCBD5E1"/></top><bottom style="thin"><color rgb="FFCBD5E1"/></bottom><diagonal/></border>
<border><left/><right/><top/><bottom style="medium"><color rgb="FF0F172A"/></bottom><diagonal/></border>
<border><left style="thin"><color rgb="FF94A3B8"/></left><right style="thin"><color rgb="FF94A3B8"/></right><top style="thin"><color rgb="FF94A3B8"/></top><bottom style="thin"><color rgb="FF94A3B8"/></bottom><diagonal/></border>
</borders>
<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
<cellXfs count="18">
<xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0" applyFont="1"/>
<xf numFmtId="0" fontId="1" fillId="5" borderId="2" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment vertical="center"/></xf>
<xf numFmtId="0" fontId="2" fillId="0" borderId="0" xfId="0" applyFont="1"/>
<xf numFmtId="0" fontId="3" fillId="2" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment vertical="center" horizontal="left"/></xf>
<xf numFmtId="0" fontId="4" fillId="0" borderId="0" xfId="0" applyFont="1"/>
<xf numFmtId="0" fontId="5" fillId="5" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="top"/></xf>
<xf numFmtId="0" fontId="6" fillId="4" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="center"/></xf>
<xf numFmtId="0" fontId="7" fillId="0" borderId="0" xfId="0" applyFont="1" applyAlignment="1"><alignment wrapText="1" vertical="top"/></xf>
<xf numFmtId="0" fontId="4" fillId="3" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" horizontal="center" vertical="center"/></xf>
<xf numFmtId="164" fontId="8" fillId="0" borderId="1" xfId="0" applyNumberFormat="1" applyFont="1" applyBorder="1" applyAlignment="1"><alignment horizontal="right" vertical="center"/></xf>
<xf numFmtId="0" fontId="8" fillId="0" borderId="1" xfId="0" applyFont="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="top"/></xf>
<xf numFmtId="0" fontId="2" fillId="0" borderId="1" xfId="0" applyFont="1" applyBorder="1" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>
<xf numFmtId="0" fontId="9" fillId="5" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="center"/></xf>
<xf numFmtId="0" fontId="10" fillId="7" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="top"/></xf>
<xf numFmtId="0" fontId="8" fillId="4" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="center"/></xf>
<xf numFmtId="0" fontId="8" fillId="0" borderId="0" xfId="0" applyFont="1" applyAlignment="1"><alignment horizontal="center" vertical="center"/></xf>
<xf numFmtId="0" fontId="4" fillId="6" borderId="1" xfId="0" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment wrapText="1" vertical="center"/></xf>
<xf numFmtId="165" fontId="6" fillId="4" borderId="1" xfId="0" applyNumberFormat="1" applyFont="1" applyFill="1" applyBorder="1" applyAlignment="1"><alignment horizontal="right" vertical="center"/></xf>
</cellXfs>
<cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
</styleSheet>"#
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_core::Document;
    use std::io::Read;

    #[test]
    fn exports_formatted_workbook_with_live_formulas() {
        let document = Document::default();
        let bytes = export_xlsx(&document).expect("xlsx export should succeed");
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).expect("valid zip");

        let mut workbook = String::new();
        archive
            .by_name("xl/workbook.xml")
            .expect("workbook")
            .read_to_string(&mut workbook)
            .expect("read workbook");
        assert!(workbook.contains("Documento"));
        assert!(workbook.contains("Cálculos"));
        assert!(workbook.contains("fullCalcOnLoad"));

        let mut sheet = String::new();
        archive
            .by_name("xl/worksheets/sheet2.xml")
            .expect("calculations sheet")
            .read_to_string(&mut sheet)
            .expect("read calculations sheet");
        assert!(sheet.contains("<f>(2 + 2)</f>"), "{sheet}");
        assert!(sheet.contains("Fórmula Excel"));

        let mut styles = String::new();
        archive
            .by_name("xl/styles.xml")
            .expect("styles")
            .read_to_string(&mut styles)
            .expect("read styles");
        assert!(styles.contains("Cambria Math"));
    }
}
