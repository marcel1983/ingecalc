// SPDX-License-Identifier: GPL-3.0-or-later

//! Out-of-process plugin host.
//!
//! The first implementation will spawn a plugin executable and speak
//! JSON-RPC 2.0 over stdio. This module is a placeholder for the host
//! lifecycle, capability checks and timeout handling.

use plugin_api::{PluginError, PluginManifest};

#[derive(Debug, Default)]
pub struct PluginHost;

impl PluginHost {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_manifest(&self, manifest: &PluginManifest) -> Result<(), PluginError> {
        manifest.validate()
    }
}
