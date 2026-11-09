// SPDX-License-Identifier: GPL-3.0-or-later

mod html;
mod latex;
mod latex_pdf;
mod mathml;
mod odt;
mod results;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Latex,
    Odt,
}

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("invalid base64 image: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("unsupported export: {0}")]
    Unsupported(String),
    #[error("{0}")]
    Latex(String),
}

pub use latex::export_latex;
pub use latex_pdf::{detect_latex_engine, export_pdf_latex, find_latex_engine};
pub use odt::export_odt;
