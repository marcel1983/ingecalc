// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(name = "ingecalc", version, about = "IngeCalc command line interface")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a MathJSON expression.
    Eval {
        /// MathJSON value, for example: '["Add", 2, 2]'
        mathjson: String,
    },
    /// Evaluate a compact ASCII-math expression.
    EvalText {
        /// Expression, for example: '2 + 3 * 4'
        expression: String,
    },
    /// Validate a .smj document.
    Validate {
        /// Path to the document.
        path: String,
    },
    /// Evaluate all math regions in document order.
    EvalDocument {
        /// Path to the .smj document.
        input: String,
    },
    /// Export a document to LaTeX, ODT or PDF.
    Export {
        /// Path to the .smj document.
        input: String,
        /// Output format: "latex", "odt" or "pdf".
        format: String,
        /// Output file path.
        output: String,
        /// Optional LaTeX engine path for PDF export.
        #[arg(long)]
        engine: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Eval { mathjson } => {
            let value: serde_json::Value = match serde_json::from_str(&mathjson) {
                Ok(value) => value,
                Err(error) => {
                    eprintln!(
                        "{}",
                        json!({"ok": false, "error": format!("invalid JSON: {error}")})
                    );
                    std::process::exit(2);
                }
            };

            let result = math_json::from_value(&value)
                .map_err(|error| error.to_string())
                .and_then(|expr| math_eval::evaluate(&expr).map_err(|error| error.to_string()));

            match result {
                Ok(value) => println!(
                    "{}",
                    json!({"ok": true, "value": value.display(), "raw": value})
                ),
                Err(error) => {
                    eprintln!("{}", json!({"ok": false, "error": error}));
                    std::process::exit(1);
                }
            }
        }
        Commands::EvalText { expression } => {
            let result = math_parse::parse(&expression)
                .map_err(|error| error.to_string())
                .and_then(|expr| math_eval::evaluate(&expr).map_err(|error| error.to_string()));

            match result {
                Ok(value) => println!(
                    "{}",
                    json!({"ok": true, "value": value.display(), "raw": value})
                ),
                Err(error) => {
                    eprintln!("{}", json!({"ok": false, "error": error}));
                    std::process::exit(1);
                }
            }
        }
        Commands::Validate { path } => match doc_json::load(&path) {
            Ok(document) => println!(
                "{}",
                json!({
                    "ok": true,
                    "title": document.metadata.title,
                    "pages": document.pages.len()
                })
            ),
            Err(error) => {
                eprintln!("{}", json!({"ok": false, "error": error.to_string()}));
                std::process::exit(1);
            }
        },
        Commands::EvalDocument { input } => {
            let document = match doc_json::load(&input) {
                Ok(document) => document,
                Err(error) => {
                    eprintln!("{}", json!({"ok": false, "error": error.to_string()}));
                    std::process::exit(1);
                }
            };

            let mut items = Vec::new();
            for page in &document.pages {
                let mut regions = page
                    .regions
                    .iter()
                    .filter_map(|region| match region {
                        doc_core::Region::Math(math) => Some(math),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                regions.sort_by(|left, right| {
                    left.y
                        .total_cmp(&right.y)
                        .then_with(|| left.x.total_cmp(&right.x))
                });
                for math in regions {
                    if !math.ascii.trim().is_empty() {
                        items.push((math.id.clone(), math.ascii.trim().to_string()));
                    }
                }
            }

            let numbers = math_eval::NumberFormat::from_settings(
                &document.settings.notation,
                document.settings.precision,
                document.settings.trim_zeros,
                &document.settings.fractions,
            );
            let constants = document
                .settings
                .custom_constants
                .iter()
                .map(|constant| (constant.name.clone(), constant.expression.clone()))
                .collect::<Vec<_>>();
            let results = math_eval::evaluate_document_with_constants_and_preferences(
                &items,
                &constants,
                &math_eval::UnitPreferences::new(
                    document.settings.units.clone(),
                    document.settings.unit_overrides.clone(),
                ),
            )
            .into_iter()
            .map(|evaluation| match evaluation.result {
                Ok(value) => json!({
                    "id": evaluation.id,
                    "value": value.display_with_format(&numbers),
                    "error": null
                }),
                Err(error) => json!({
                    "id": evaluation.id,
                    "value": null,
                    "error": error.to_string()
                }),
            })
            .collect::<Vec<_>>();

            println!("{}", json!({"ok": true, "results": results}));
        }
        Commands::Export {
            input,
            format,
            output,
            engine,
        } => {
            let document = match doc_json::load(&input) {
                Ok(document) => document,
                Err(error) => {
                    eprintln!("{}", json!({"ok": false, "error": error.to_string()}));
                    std::process::exit(1);
                }
            };

            let result = match format.to_ascii_lowercase().as_str() {
                "latex" | "tex" => export::export_latex(&document, std::path::Path::new(&output)),
                "odt" => export::export_odt(&document).and_then(|bytes| {
                    std::fs::write(&output, bytes).map_err(export::ExportError::from)
                }),
                "pdf" | "pdf-latex" => export::export_pdf_latex(
                    &document,
                    std::path::Path::new(&output),
                    engine.as_deref().map(std::path::Path::new),
                ),
                other => {
                    eprintln!(
                        "{}",
                        json!({"ok": false, "error": format!("unsupported format: {other}")})
                    );
                    std::process::exit(2);
                }
            };

            match result {
                Ok(()) => println!(
                    "{}",
                    json!({"ok": true, "format": format, "output": output})
                ),
                Err(error) => {
                    eprintln!("{}", json!({"ok": false, "error": error.to_string()}));
                    std::process::exit(1);
                }
            }
        }
    }
}
