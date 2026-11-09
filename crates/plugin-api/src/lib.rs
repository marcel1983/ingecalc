// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PLUGIN_API_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: String,
    pub entry: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("unsupported plugin API version: {0}")]
    ApiVersion(String),
    #[error("plugin manifest is invalid: {0}")]
    Manifest(String),
    #[error("plugin failed: {0}")]
    Runtime(String),
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.api_version != PLUGIN_API_VERSION {
            return Err(PluginError::ApiVersion(self.api_version.clone()));
        }
        if self.id.is_empty() || self.entry.is_empty() {
            return Err(PluginError::Manifest(
                "id and entry must not be empty".into(),
            ));
        }
        Ok(())
    }
}
