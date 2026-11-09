// SPDX-License-Identifier: GPL-3.0-or-later

use doc_core::{is_supported_format, Document, FORMAT_NAME, FORMAT_VERSION, LEGACY_FORMAT_NAME};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocJsonError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported document format: {0}")]
    Format(String),
    #[error("unsupported document version: {0}")]
    Version(u32),
}

pub fn load(path: impl AsRef<Path>) -> Result<Document, DocJsonError> {
    let text = fs::read_to_string(path)?;
    let mut document: Document = serde_json::from_str(&text)?;

    if !is_supported_format(&document.format) {
        return Err(DocJsonError::Format(document.format));
    }
    if document.format == LEGACY_FORMAT_NAME {
        document.format = FORMAT_NAME.to_string();
    }
    if document.version != FORMAT_VERSION {
        return Err(DocJsonError::Version(document.version));
    }

    Ok(document)
}

pub fn save(path: impl AsRef<Path>, document: &Document) -> Result<(), DocJsonError> {
    let text = serde_json::to_string_pretty(document)?;
    fs::write(path, text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_legacy_format_name() {
        assert!(is_supported_format("ingecalc-json"));
        assert!(is_supported_format("mathdoc-json"));
        assert!(!is_supported_format("otro-formato"));
    }

    #[test]
    fn upgrades_legacy_format_on_load() {
        let mut value = serde_json::to_value(Document::default()).expect("document to JSON");
        value["format"] = serde_json::Value::String("mathdoc-json".into());
        let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("target")
            .join("test-output")
            .join("doc-json");
        fs::create_dir_all(&directory).expect("create test directory");
        let path = directory.join(format!(
            "ingecalc-legacy-{}-{}.smj",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::write(&path, serde_json::to_string(&value).expect("serialize")).expect("write");
        let document = load(&path).expect("legacy document");
        let _ = fs::remove_file(&path);
        assert_eq!(document.format, FORMAT_NAME);
    }

    #[test]
    fn loads_documents_without_page_setup() {
        let mut value = serde_json::to_value(Document::default()).expect("document to JSON");
        value
            .get_mut("settings")
            .and_then(|settings| settings.as_object_mut())
            .expect("settings object")
            .remove("page");

        let document: Document = serde_json::from_value(value).expect("legacy document");
        assert_eq!(document.settings.page.margin_top, 48.0);
        assert_eq!(document.settings.page.margin_left, 48.0);
        assert!(document.settings.page.header.is_empty());
        assert!(document.settings.page.footer.is_empty());
    }
}
