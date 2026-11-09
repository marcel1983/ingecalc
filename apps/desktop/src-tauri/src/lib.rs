// SPDX-License-Identifier: GPL-3.0-or-later

use doc_core::{Document, Region};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Debug, Serialize)]
struct EvalResponse {
    value: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct DocumentEvalResult {
    id: String,
    value: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn evaluate_math(text: String) -> EvalResponse {
    let result = math_parse::parse(&text)
        .map_err(|error| error.to_string())
        .and_then(|expr| math_eval::evaluate(&expr).map_err(|error| error.to_string()));

    match result {
        Ok(value) => EvalResponse {
            value: Some(value.display()),
            error: None,
        },
        Err(error) => EvalResponse {
            value: None,
            error: Some(error),
        },
    }
}

#[tauri::command]
fn evaluate_document(document: Document) -> Vec<DocumentEvalResult> {
    let items = document_math_items(&document);
    let preferences = math_eval::UnitPreferences::new(
        document.settings.units.clone(),
        document.settings.unit_overrides.clone(),
    );
    let constants = document
        .settings
        .custom_constants
        .iter()
        .map(|constant| (constant.name.clone(), constant.expression.clone()))
        .collect::<Vec<_>>();
    let numbers = math_eval::NumberFormat::from_settings(
        &document.settings.notation,
        document.settings.precision,
        document.settings.trim_zeros,
        &document.settings.fractions,
    );

    math_eval::evaluate_document_with_constants_and_preferences(&items, &constants, &preferences)
        .into_iter()
        .map(|evaluation| match evaluation.result {
            Ok(value) => DocumentEvalResult {
                id: evaluation.id,
                value: Some(value.display_with_format(&numbers)),
                error: None,
            },
            Err(error) => DocumentEvalResult {
                id: evaluation.id,
                value: None,
                error: Some(error.to_string()),
            },
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SurfaceRequest {
    document: Document,
    expression: String,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    samples: usize,
}

#[derive(Debug, Serialize)]
struct SurfaceResult {
    grid: Vec<Vec<Option<f64>>>,
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
}

#[tauri::command]
#[allow(clippy::needless_range_loop)]
fn sample_surface(request: SurfaceRequest) -> SurfaceResult {
    if !request.x_min.is_finite()
        || !request.x_max.is_finite()
        || !request.y_min.is_finite()
        || !request.y_max.is_finite()
        || request.x_max <= request.x_min
        || request.y_max <= request.y_min
    {
        return SurfaceResult {
            grid: Vec::new(),
            error: Some("invalid surface domain".into()),
            unit: None,
        };
    }

    let samples = request.samples.clamp(8, 80);
    let expression = match parse_plot_expression(&request.expression) {
        Ok(expression) => expression,
        Err(error) => {
            return SurfaceResult {
                grid: Vec::new(),
                error: Some(error),
                unit: None,
            }
        }
    };

    let items = document_math_items(&request.document);
    let (_, environment) = math_eval::evaluate_document_with_environment(&items);
    let mut local = environment;
    let mut grid = vec![vec![None; samples + 1]; samples + 1];
    let mut last_error = String::new();
    let mut unit: Option<String> = None;

    for row in 0..=samples {
        let y = request.y_min + (request.y_max - request.y_min) * row as f64 / samples as f64;
        for column in 0..=samples {
            let x =
                request.x_min + (request.x_max - request.x_min) * column as f64 / samples as f64;
            local.set("x", math_numeric::Value::Real(x));
            local.set("y", math_numeric::Value::Real(y));
            match numeric_value_with_unit(&expression, &mut local) {
                Ok((value, sample_unit)) if value.is_finite() => {
                    if let Some(sample_unit) = sample_unit {
                        if let Some(existing) = &unit {
                            if existing != &sample_unit {
                                last_error = "the surface mixes units".into();
                            }
                        } else {
                            unit = Some(sample_unit);
                        }
                    }
                    grid[row][column] = Some(value);
                }
                Ok(_) => last_error = "non-finite surface value".into(),
                Err(error) => last_error = error,
            }
        }
    }

    SurfaceResult {
        error: if grid.iter().all(|row| row.iter().all(Option::is_none)) && !last_error.is_empty() {
            Some(last_error)
        } else {
            None
        },
        grid,
        unit,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlotRequest {
    document: Document,
    series: Vec<PlotSeriesRequest>,
    x_min: f64,
    x_max: f64,
    y_min: Option<f64>,
    y_max: Option<f64>,
    samples: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlotSeriesRequest {
    kind: String,
    expression: String,
    #[serde(default)]
    expression2: String,
    #[serde(default = "default_parameter_min")]
    parameter_min: f64,
    #[serde(default = "default_parameter_max")]
    parameter_max: f64,
}

#[derive(Debug, Default, Serialize)]
struct PlotSampleResult {
    paths: Vec<Vec<[f64; 2]>>,
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    x_unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y_unit: Option<String>,
}

fn default_parameter_min() -> f64 {
    -10.0
}

fn default_parameter_max() -> f64 {
    10.0
}

#[tauri::command]
fn sample_plot(request: PlotRequest) -> Vec<PlotSampleResult> {
    if !request.x_min.is_finite() || !request.x_max.is_finite() || request.x_max <= request.x_min {
        return request
            .series
            .into_iter()
            .map(|_| PlotSampleResult {
                paths: Vec::new(),
                error: Some("invalid plot domain".into()),
                ..Default::default()
            })
            .collect();
    }

    let samples = request.samples.clamp(2, 2000);
    let items = document_math_items(&request.document);
    let (_, environment) = math_eval::evaluate_document_with_environment(&items);

    request
        .series
        .into_iter()
        .map(|series| match series.kind.as_str() {
            "parametric" => sample_parametric(&series, &environment),
            "implicit" => sample_implicit(
                &series,
                &environment,
                request.x_min,
                request.x_max,
                request.y_min,
                request.y_max,
            ),
            _ => sample_explicit(&series, &environment, request.x_min, request.x_max, samples),
        })
        .collect()
}

fn sample_explicit(
    series: &PlotSeriesRequest,
    environment: &math_eval::Environment,
    x_min: f64,
    x_max: f64,
    samples: usize,
) -> PlotSampleResult {
    let expression = match parse_plot_expression(&series.expression) {
        Ok(expression) => expression,
        Err(error) => return plot_error(error),
    };

    let (points, error, unit) = sample_path(&expression, "x", x_min, x_max, samples, environment);
    PlotSampleResult {
        paths: if points.is_empty() {
            Vec::new()
        } else {
            vec![points]
        },
        error: if error.is_empty() { None } else { Some(error) },
        unit,
        ..Default::default()
    }
}

fn sample_parametric(
    series: &PlotSeriesRequest,
    environment: &math_eval::Environment,
) -> PlotSampleResult {
    let x_expression = match parse_plot_expression(&series.expression) {
        Ok(expression) => expression,
        Err(error) => return plot_error(error),
    };
    let y_expression = match parse_plot_expression(&series.expression2) {
        Ok(expression) => expression,
        Err(error) => return plot_error(error),
    };
    if !series.parameter_min.is_finite()
        || !series.parameter_max.is_finite()
        || series.parameter_max <= series.parameter_min
    {
        return plot_error("invalid parametric range".into());
    }

    let samples = 480;
    let mut points = Vec::with_capacity(samples);
    let mut local = environment.clone();
    let mut last_error = String::new();
    let mut x_unit: Option<String> = None;
    let mut y_unit: Option<String> = None;

    for index in 0..samples {
        let t = series.parameter_min
            + (series.parameter_max - series.parameter_min) * index as f64 / (samples - 1) as f64;
        local.set("t", math_numeric::Value::Real(t));
        let (x, sample_x_unit) = match numeric_value_with_unit(&x_expression, &mut local) {
            Ok(value) => value,
            Err(error) => {
                last_error = error;
                continue;
            }
        };
        let (y, sample_y_unit) = match numeric_value_with_unit(&y_expression, &mut local) {
            Ok(value) => value,
            Err(error) => {
                last_error = error;
                continue;
            }
        };
        if let Some(sample) = sample_x_unit {
            if x_unit.as_ref().is_some_and(|existing| existing != &sample) {
                last_error = "x(t) mixes units".into();
            } else {
                x_unit = Some(sample);
            }
        }
        if let Some(sample) = sample_y_unit {
            if y_unit.as_ref().is_some_and(|existing| existing != &sample) {
                last_error = "y(t) mixes units".into();
            } else {
                y_unit = Some(sample);
            }
        }
        if x.is_finite() && y.is_finite() {
            points.push([x, y]);
        }
    }

    let error = if points.is_empty() && !last_error.is_empty() {
        Some(last_error)
    } else {
        None
    };
    PlotSampleResult {
        paths: if points.is_empty() {
            Vec::new()
        } else {
            vec![points]
        },
        error,
        x_unit,
        y_unit,
        ..Default::default()
    }
}

fn sample_implicit(
    series: &PlotSeriesRequest,
    environment: &math_eval::Environment,
    x_min: f64,
    x_max: f64,
    y_min: Option<f64>,
    y_max: Option<f64>,
) -> PlotSampleResult {
    let expression = match parse_plot_equation(&series.expression) {
        Ok(expression) => expression,
        Err(error) => return plot_error(error),
    };

    let x_span = x_max - x_min;
    let center = (x_min + x_max) / 2.0;
    let y_min = y_min
        .filter(|value| value.is_finite())
        .unwrap_or(center - x_span / 2.0);
    let y_max = y_max
        .filter(|value| value.is_finite())
        .unwrap_or(center + x_span / 2.0);
    if y_max <= y_min {
        return plot_error("invalid implicit y range".into());
    }

    let grid = 80usize;
    let mut values = vec![vec![0.0f64; grid + 1]; grid + 1];
    let mut local = environment.clone();
    let mut last_error = String::new();

    for (row, row_values) in values.iter_mut().enumerate() {
        let y = y_max - (y_max - y_min) * row as f64 / grid as f64;
        for (column, cell) in row_values.iter_mut().enumerate() {
            let x = x_min + x_span * column as f64 / grid as f64;
            local.set("x", math_numeric::Value::Real(x));
            local.set("y", math_numeric::Value::Real(y));
            match numeric_value(&expression, &mut local) {
                Ok(value) => *cell = value,
                Err(error) => {
                    last_error = error;
                    *cell = f64::NAN;
                }
            }
        }
    }

    let mut paths = Vec::new();
    for row in 0..grid {
        let y0 = y_max - (y_max - y_min) * row as f64 / grid as f64;
        let y1 = y_max - (y_max - y_min) * (row + 1) as f64 / grid as f64;
        for column in 0..grid {
            let x0 = x_min + x_span * column as f64 / grid as f64;
            let x1 = x_min + x_span * (column + 1) as f64 / grid as f64;
            let bottom_left = values[row][column];
            let bottom_right = values[row][column + 1];
            let top_right = values[row + 1][column + 1];
            let top_left = values[row + 1][column];

            if [bottom_left, bottom_right, top_right, top_left]
                .iter()
                .any(|value| !value.is_finite())
            {
                continue;
            }

            let mut crossings = Vec::new();
            if crosses(bottom_left, bottom_right) {
                crossings.push(crossing(bottom_left, bottom_right, x0, y0, x1, y0));
            }
            if crosses(bottom_right, top_right) {
                crossings.push(crossing(bottom_right, top_right, x1, y0, x1, y1));
            }
            if crosses(top_right, top_left) {
                crossings.push(crossing(top_right, top_left, x1, y1, x0, y1));
            }
            if crosses(top_left, bottom_left) {
                crossings.push(crossing(top_left, bottom_left, x0, y1, x0, y0));
            }

            let mut index = 0;
            while index + 1 < crossings.len() {
                paths.push(vec![crossings[index], crossings[index + 1]]);
                index += 2;
            }
        }
    }

    PlotSampleResult {
        error: if paths.is_empty() && !last_error.is_empty() {
            Some(last_error)
        } else {
            None
        },
        paths,
        ..Default::default()
    }
}

fn parse_plot_expression(text: &str) -> Result<math_ast::Expr, String> {
    let text = text.trim();
    if text.is_empty() {
        return Err("empty plot expression".into());
    }
    math_parse::parse(text).map_err(|error| error.to_string())
}

fn parse_plot_equation(text: &str) -> Result<math_ast::Expr, String> {
    if let Some((left, right)) = text.split_once('=') {
        let left = parse_plot_expression(left)?;
        let right = parse_plot_expression(right)?;
        Ok(math_ast::Expr::binary(math_ast::BinaryOp::Sub, left, right))
    } else {
        parse_plot_expression(text)
    }
}

fn sample_path(
    expression: &math_ast::Expr,
    variable: &str,
    from: f64,
    to: f64,
    samples: usize,
    environment: &math_eval::Environment,
) -> (Vec<[f64; 2]>, String, Option<String>) {
    let mut points = Vec::with_capacity(samples);
    let mut local = environment.clone();
    let mut last_error = String::new();
    let mut unit: Option<String> = None;

    for index in 0..samples {
        let value = from + (to - from) * index as f64 / (samples - 1) as f64;
        local.set(variable, math_numeric::Value::Real(value));
        match numeric_value_with_unit(expression, &mut local) {
            Ok((result, sample_unit)) if result.is_finite() => {
                if let Some(sample_unit) = sample_unit {
                    if let Some(existing) = &unit {
                        if existing != &sample_unit {
                            last_error = "the series mixes units".into();
                        }
                    } else {
                        unit = Some(sample_unit);
                    }
                }
                points.push([value, result]);
            }
            Ok(_) => last_error = "non-finite plot value".into(),
            Err(error) => last_error = error,
        }
    }

    (points, last_error, unit)
}

fn numeric_value(
    expression: &math_ast::Expr,
    environment: &mut math_eval::Environment,
) -> Result<f64, String> {
    numeric_value_with_unit(expression, environment).map(|(value, _)| value)
}

fn numeric_value_with_unit(
    expression: &math_ast::Expr,
    environment: &mut math_eval::Environment,
) -> Result<(f64, Option<String>), String> {
    match math_eval::evaluate_with(expression, environment) {
        Ok(math_numeric::Value::Quantity(quantity)) => {
            Ok((quantity.display_value(), Some(quantity.unit.symbol.clone())))
        }
        Ok(value) => value
            .scalar()
            .map(|value| (value, None))
            .ok_or_else(|| "plot expression must return a real scalar".into()),
        Err(error) => Err(error.to_string()),
    }
}

fn crosses(left: f64, right: f64) -> bool {
    left == 0.0 || right == 0.0 || (left < 0.0) != (right < 0.0)
}

fn crossing(left: f64, right: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> [f64; 2] {
    let denominator = right - left;
    let fraction = if denominator.abs() < 1e-12 {
        0.5
    } else {
        (-left / denominator).clamp(0.0, 1.0)
    };
    [x0 + (x1 - x0) * fraction, y0 + (y1 - y0) * fraction]
}

fn plot_error(error: String) -> PlotSampleResult {
    PlotSampleResult {
        paths: Vec::new(),
        error: Some(error),
        ..Default::default()
    }
}

fn document_math_items(document: &Document) -> Vec<(String, String)> {
    let mut items = Vec::new();

    for page in &document.pages {
        let mut regions = page
            .regions
            .iter()
            .filter_map(|region| match region {
                Region::Math(math) => Some(math),
                _ => None,
            })
            .collect::<Vec<_>>();
        regions.sort_by(|left, right| {
            left.y
                .total_cmp(&right.y)
                .then_with(|| left.x.total_cmp(&right.x))
        });

        for math in regions {
            let text = math.ascii.trim();
            if !text.is_empty() {
                items.push((math.id.clone(), text.to_string()));
            }
        }
    }

    items
}

#[tauri::command]
fn new_document() -> Document {
    Document::default()
}

#[tauri::command]
fn load_document(path: String) -> Result<Document, String> {
    doc_json::load(&path).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_document(path: String, document: Document) -> Result<(), String> {
    doc_json::save(&path, &document).map_err(|error| error.to_string())
}

#[tauri::command]
fn export_document(document: Document, format: String, path: String) -> Result<(), String> {
    let result = match format.to_ascii_lowercase().as_str() {
        "latex" | "tex" => export::export_latex(&document, std::path::Path::new(&path)),
        "odt" => export::export_odt(&document)
            .and_then(|bytes| std::fs::write(&path, bytes).map_err(export::ExportError::from)),
        other => return Err(format!("unsupported export format: {other}")),
    };

    result.map_err(|error| error.to_string())
}

#[tauri::command]
fn save_pdf(path: String, data: String) -> Result<(), String> {
    save_binary(path, data)
}

#[tauri::command]
fn save_binary(path: String, data: String) -> Result<(), String> {
    use base64::Engine as _;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

#[tauri::command]
fn read_image(path: String) -> Result<String, String> {
    use base64::Engine as _;

    let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
    let extension = std::path::Path::new(&path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    let mime = match extension.as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredDraft {
    saved_at: u64,
    document: Document,
}

fn env_path(primary: &str, legacy: &str) -> Option<PathBuf> {
    std::env::var_os(primary)
        .or_else(|| std::env::var_os(legacy))
        .map(PathBuf::from)
}

fn draft_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if let Some(directory) = env_path("INGECALC_DRAFT_DIR", "MATHDOC_DRAFT_DIR") {
        return Ok(directory);
    }

    app.path()
        .app_local_data_dir()
        .map_err(|error| format!("No se pudo resolver el directorio de borradores: {error}"))
}

fn draft_path(directory: &Path) -> PathBuf {
    directory.join("draft.smj")
}

fn write_draft_file(directory: &Path, document: &Document) -> Result<u64, String> {
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let saved_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis() as u64;
    let draft = StoredDraft {
        saved_at,
        document: document.clone(),
    };
    let json = serde_json::to_string_pretty(&draft).map_err(|error| error.to_string())?;
    let path = draft_path(directory);
    let temporary = directory.join("draft.smj.tmp");

    std::fs::write(&temporary, json).map_err(|error| error.to_string())?;
    let _ = std::fs::remove_file(&path);
    std::fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
    Ok(saved_at)
}

fn read_draft_file(directory: &Path) -> Result<Option<StoredDraft>, String> {
    let path = draft_path(directory);
    if !path.is_file() {
        return Ok(None);
    }

    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let draft = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    Ok(Some(draft))
}

fn clear_draft_file(directory: &Path) -> Result<(), String> {
    let path = draft_path(directory);
    if path.is_file() {
        std::fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn save_draft(app: tauri::AppHandle, document: Document) -> Result<u64, String> {
    write_draft_file(&draft_directory(&app)?, &document)
}

#[tauri::command]
fn load_draft(app: tauri::AppHandle) -> Result<Option<StoredDraft>, String> {
    read_draft_file(&draft_directory(&app)?)
}

#[tauri::command]
fn clear_draft(app: tauri::AppHandle) -> Result<(), String> {
    clear_draft_file(&draft_directory(&app)?)
}

#[tauri::command]
fn detect_latex_engine() -> Option<String> {
    export::detect_latex_engine()
}

#[tauri::command]
async fn export_pdf_latex(
    document: Document,
    path: String,
    engine: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        export::export_pdf_latex(
            &document,
            Path::new(&path),
            engine.as_deref().map(Path::new),
        )
        .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("La tarea de LaTeX falló: {error}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_directory = env_path("INGECALC_WEBVIEW_DATA_DIR", "MATHDOC_WEBVIEW_DATA_DIR")
                .unwrap_or_else(|| {
                    app.path()
                        .app_local_data_dir()
                        .unwrap_or_else(|_| std::env::temp_dir())
                        .join("webview2")
                });

            let url = if tauri::is_dev() {
                WebviewUrl::External(
                    "http://localhost:1420"
                        .parse()
                        .expect("valid development URL"),
                )
            } else {
                WebviewUrl::App("index.html".into())
            };

            WebviewWindowBuilder::new(app, "main", url)
                .title("IngeCalc")
                .inner_size(1280.0, 900.0)
                .min_inner_size(900.0, 600.0)
                .resizable(true)
                .data_directory(data_directory)
                .build()?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            evaluate_math,
            evaluate_document,
            sample_plot,
            sample_surface,
            new_document,
            load_document,
            save_document,
            export_document,
            save_pdf,
            save_binary,
            read_image,
            detect_latex_engine,
            export_pdf_latex,
            save_draft,
            load_draft,
            clear_draft
        ])
        .run(tauri::generate_context!())
        .expect("error while running the IngeCalc desktop application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn saves_pdf_bytes() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("pdf");
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("test.pdf");

        let data = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4\n%%EOF\n");
        save_pdf(path.to_string_lossy().to_string(), data).expect("save PDF");

        let bytes = std::fs::read(&path).expect("read PDF");
        assert_eq!(&bytes[..5], b"%PDF-");

        std::fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn evaluates_math_expression() {
        let response = evaluate_math("2+2".into());
        assert_eq!(response.value.as_deref(), Some("4"));
        assert!(response.error.is_none());
    }

    #[test]
    fn evaluates_document_with_variables_units_and_matrices() {
        let mut document = Document::default();
        let region = |id: &str, y: f64, ascii: &str| {
            Region::Math(doc_core::MathRegion {
                id: id.into(),
                x: 60.0,
                y,
                w: 360.0,
                h: 48.0,
                source: ascii.into(),
                ascii: ascii.into(),
                pinned: false,
            })
        };
        document.pages[0].regions = vec![
            region("assign", 10.0, "x := 3"),
            region("variable", 60.0, "x^2"),
            region("unit", 110.0, "2\"m\" + 30\"cm\""),
            region("matrix", 160.0, "((1,2),(3,4))"),
            region("det", 210.0, "det([1,2;3,4])"),
        ];

        let results = evaluate_document(document);
        let value = |id: &str| {
            results
                .iter()
                .find(|result| result.id == id)
                .and_then(|result| result.value.as_deref())
        };

        assert_eq!(value("assign"), Some("3"));
        assert_eq!(value("variable"), Some("9"));
        assert_eq!(value("unit"), Some("2.3 m"));
        assert_eq!(value("matrix"), Some("[1, 2; 3, 4]"));
        assert_eq!(value("det"), Some("-2"));
    }

    #[test]
    fn evaluates_document_with_functions_and_forward_references() {
        let mut document = Document::default();
        let region = |id: &str, y: f64, ascii: &str| {
            Region::Math(doc_core::MathRegion {
                id: id.into(),
                x: 60.0,
                y,
                w: 360.0,
                h: 48.0,
                source: ascii.into(),
                ascii: ascii.into(),
                pinned: false,
            })
        };
        document.pages[0].regions = vec![
            region("use", 10.0, "f(3) + a"),
            region("function", 60.0, "f(x) := x^2"),
            region("variable", 110.0, "a := 1"),
        ];

        let results = evaluate_document(document);
        let value = |id: &str| {
            results
                .iter()
                .find(|result| result.id == id)
                .and_then(|result| result.value.as_deref())
        };

        assert_eq!(value("function"), Some("f(x)"));
        assert_eq!(value("use"), Some("10"));
    }

    #[test]
    fn samples_plot_expression_with_document_variables() {
        let mut document = Document::default();
        document.pages[0].regions = vec![Region::Math(doc_core::MathRegion {
            id: "a".into(),
            x: 60.0,
            y: 60.0,
            w: 360.0,
            h: 48.0,
            source: "a := 2".into(),
            ascii: "a := 2".into(),
            pinned: false,
        })];

        let results = sample_plot(PlotRequest {
            document,
            series: vec![PlotSeriesRequest {
                kind: "explicit".into(),
                expression: "a*sin(x)".into(),
                expression2: String::new(),
                parameter_min: -10.0,
                parameter_max: 10.0,
            }],
            x_min: 0.0,
            x_max: std::f64::consts::PI,
            y_min: None,
            y_max: None,
            samples: 5,
        });

        assert_eq!(results.len(), 1);
        assert!(results[0].error.is_none());
        assert_eq!(results[0].paths.len(), 1);
        assert_eq!(results[0].paths[0].len(), 5);
        assert!((results[0].paths[0][0][1]).abs() < 1e-9);
        assert!(results[0].paths[0][2][1] > 1.9);
    }

    #[test]
    fn samples_parametric_and_implicit_plots() {
        let document = Document::default();
        let parametric = sample_plot(PlotRequest {
            document: document.clone(),
            series: vec![PlotSeriesRequest {
                kind: "parametric".into(),
                expression: "cos(t)".into(),
                expression2: "sin(t)".into(),
                parameter_min: 0.0,
                parameter_max: std::f64::consts::TAU,
            }],
            x_min: -1.0,
            x_max: 1.0,
            y_min: None,
            y_max: None,
            samples: 100,
        });
        assert!(parametric[0].error.is_none());
        assert_eq!(parametric[0].paths.len(), 1);
        assert!((parametric[0].paths[0][0][0] - 1.0).abs() < 1e-9);

        let implicit = sample_plot(PlotRequest {
            document,
            series: vec![PlotSeriesRequest {
                kind: "implicit".into(),
                expression: "x^2+y^2-1".into(),
                expression2: String::new(),
                parameter_min: -1.0,
                parameter_max: 1.0,
            }],
            x_min: -2.0,
            x_max: 2.0,
            y_min: Some(-2.0),
            y_max: Some(2.0),
            samples: 100,
        });
        assert!(implicit[0].error.is_none());
        assert!(!implicit[0].paths.is_empty());
    }

    #[test]
    fn reads_image_as_data_url() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("image");
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("pixel.png");
        std::fs::write(&path, b"\x89PNG\r\n\x1a\n").expect("write image");

        let data_url = read_image(path.to_string_lossy().to_string()).expect("read image");
        assert!(data_url.starts_with("data:image/png;base64,"));

        std::fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn writes_reads_and_clears_draft() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("draft");
        std::fs::remove_dir_all(&directory).ok();

        let mut document = Document::default();
        document.metadata.title = "Borrador".into();

        let saved_at = write_draft_file(&directory, &document).expect("write draft");
        let loaded = read_draft_file(&directory)
            .expect("read draft")
            .expect("draft exists");
        assert_eq!(loaded.saved_at, saved_at);
        assert_eq!(loaded.document.metadata.title, "Borrador");

        document.metadata.title = "Borrador 2".into();
        let saved_again = write_draft_file(&directory, &document).expect("overwrite draft");
        let loaded_again = read_draft_file(&directory)
            .expect("read draft again")
            .expect("draft exists again");
        assert!(saved_again >= saved_at);
        assert_eq!(loaded_again.document.metadata.title, "Borrador 2");

        clear_draft_file(&directory).expect("clear draft");
        assert!(read_draft_file(&directory)
            .expect("read cleared draft")
            .is_none());

        std::fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn saves_and_loads_document() {
        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("document");
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("test.smj");
        let path_string = path.to_string_lossy().to_string();

        let document = Document::default();
        save_document(path_string.clone(), document.clone()).expect("save document");
        let loaded = load_document(path_string).expect("load document");

        assert_eq!(loaded.metadata.title, document.metadata.title);
        assert_eq!(loaded.pages.len(), document.pages.len());

        std::fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn samples_a_surface_grid() {
        let result = sample_surface(SurfaceRequest {
            document: Document::default(),
            expression: "x^2 + y^2".into(),
            x_min: -1.0,
            x_max: 1.0,
            y_min: -1.0,
            y_max: 1.0,
            samples: 8,
        });

        assert!(result.error.is_none());
        assert_eq!(result.grid.len(), 9);
        assert_eq!(result.grid[0].len(), 9);
        assert!(result.grid[4][4].expect("center value").abs() < 1e-9);
        assert!(result.grid[0][0].expect("corner value") > 1.9);
    }

    #[test]
    fn samples_a_plot_with_units() {
        let result = sample_plot(PlotRequest {
            document: Document::default(),
            series: vec![PlotSeriesRequest {
                kind: "explicit".into(),
                expression: "x * 1 m".into(),
                expression2: String::new(),
                parameter_min: -10.0,
                parameter_max: 10.0,
            }],
            x_min: 0.0,
            x_max: 2.0,
            y_min: None,
            y_max: None,
            samples: 5,
        });

        assert_eq!(result.len(), 1);
        assert!(result[0].error.is_none());
        assert_eq!(result[0].unit.as_deref(), Some("m"));
        assert_eq!(result[0].paths[0].len(), 5);
        assert!((result[0].paths[0][0][1]).abs() < 1e-9);
        assert!((result[0].paths[0][4][1] - 2.0).abs() < 1e-9);
    }
}
