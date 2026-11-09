// SPDX-License-Identifier: GPL-3.0-or-later

use doc_core::Document;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{export_latex, ExportError};

fn latex_engine_override() -> Option<std::ffi::OsString> {
    std::env::var_os("INGECALC_LATEX_ENGINE").or_else(|| std::env::var_os("MATHDOC_LATEX_ENGINE"))
}

/// Locate a LaTeX engine using the environment, PATH and common MiKTeX paths.
pub fn find_latex_engine() -> Option<PathBuf> {
    if let Some(value) = latex_engine_override() {
        let path = PathBuf::from(value.to_string_lossy().trim_matches('"'));
        if path.is_file() {
            return Some(path);
        }
    }

    let extension = if cfg!(windows) { ".exe" } else { "" };

    for name in ["xelatex", "lualatex", "pdflatex"] {
        if let Some(path) = find_in_path(&format!("{name}{extension}")) {
            return Some(path);
        }
    }

    #[cfg(windows)]
    {
        let mut roots = Vec::new();
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            roots.push(PathBuf::from(local).join("Programs").join("MiKTeX"));
        }
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            roots.push(PathBuf::from(program_files).join("MiKTeX"));
        }
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            roots.push(PathBuf::from(program_files_x86).join("MiKTeX"));
        }

        for root in roots {
            for name in ["xelatex.exe", "lualatex.exe", "pdflatex.exe"] {
                let candidate = root.join("miktex").join("bin").join("x64").join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

pub fn detect_latex_engine() -> Option<String> {
    find_latex_engine().map(|path| path.to_string_lossy().to_string())
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn latex_log_tail(directory: &Path) -> String {
    let log = std::fs::read_to_string(directory.join("document.log")).unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    let start = lines.len().saturating_sub(40);
    lines[start..].join("\n")
}

fn compile_latex(engine: &Path, directory: &Path) -> Result<(), ExportError> {
    let mut command = Command::new(engine);
    command
        .current_dir(directory)
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg("-no-shell-escape")
        .arg("-output-directory")
        .arg(directory)
        .arg("document.tex")
        .env("TEMP", directory)
        .env("TMP", directory);

    if let Some(parent) = engine.parent() {
        let mut paths = vec![parent.to_path_buf()];
        if let Some(existing) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&existing));
        }
        if let Ok(joined) = std::env::join_paths(paths) {
            command.env("PATH", joined);
        }
    }

    for pass in 1..=2 {
        let output = command.output().map_err(|error| {
            ExportError::Latex(format!("No se pudo ejecutar {}: {error}", engine.display()))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExportError::Latex(format!(
                "LaTeX falló en la pasada {pass} (código {}).\n{}\n{}",
                output.status.code().unwrap_or(-1),
                latex_log_tail(directory),
                stderr.trim()
            )));
        }
    }

    Ok(())
}

/// Export a document to PDF by writing LaTeX and compiling it twice with the
/// selected engine (or an auto-detected one).
pub fn export_pdf_latex(
    document: &Document,
    output: &Path,
    engine: Option<&Path>,
) -> Result<(), ExportError> {
    let engine = engine
        .map(Path::to_path_buf)
        .or_else(find_latex_engine)
        .ok_or_else(|| {
            ExportError::Latex(
                "No se encontró XeLaTeX, LuaLaTeX ni pdfLaTeX. Instala MiKTeX/TeX Live o define INGECALC_LATEX_ENGINE (o MATHDOC_LATEX_ENGINE).".into(),
            )
        })?;

    if !engine.is_file() {
        return Err(ExportError::Latex(format!(
            "El motor LaTeX configurado no existe: {}",
            engine.display()
        )));
    }

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ExportError::Latex(error.to_string()))?
        .as_millis();
    let output = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| ExportError::Latex(error.to_string()))?
            .join(output)
    };
    let parent = output
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let preferred = parent.join(format!(".ingecalc-latex-{}-{stamp}", std::process::id()));
    let fallback =
        std::env::temp_dir().join(format!("ingecalc-latex-{}-{stamp}", std::process::id()));

    let directory = if std::fs::create_dir_all(&preferred).is_ok() {
        preferred
    } else {
        std::fs::create_dir_all(&fallback)?;
        fallback
    };

    let result = (|| -> Result<(), ExportError> {
        let tex = directory.join("document.tex");
        export_latex(document, &tex)?;
        compile_latex(&engine, &directory)?;

        let pdf = directory.join("document.pdf");
        if !pdf.is_file() {
            return Err(ExportError::Latex(format!(
                "LaTeX no generó el PDF.\n{}",
                latex_log_tail(&directory)
            )));
        }

        std::fs::copy(&pdf, &output)?;
        Ok(())
    })();

    let _ = std::fs::remove_dir_all(&directory);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_pdf_when_engine_is_configured() {
        if latex_engine_override().is_none() {
            return;
        }

        let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("latex-pdf-export");
        std::fs::create_dir_all(&directory).expect("temporary directory");
        let path = directory.join("vector.pdf");

        export_pdf_latex(&Document::default(), &path, None).expect("vector PDF");

        let bytes = std::fs::read(&path).expect("read vector PDF");
        assert_eq!(&bytes[..5], b"%PDF-");
        assert!(bytes.len() > 1000);

        std::fs::remove_dir_all(directory).ok();
    }
}
