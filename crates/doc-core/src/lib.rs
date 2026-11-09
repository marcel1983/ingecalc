// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const FORMAT_NAME: &str = "ingecalc-json";
pub const LEGACY_FORMAT_NAME: &str = "mathdoc-json";
pub const FORMAT_VERSION: u32 = 1;

pub fn is_supported_format(format: &str) -> bool {
    format == FORMAT_NAME || format == LEGACY_FORMAT_NAME
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub format: String,
    pub version: u32,
    pub metadata: Metadata,
    pub settings: Settings,
    pub pages: Vec<Page>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub language: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomConstant {
    pub name: String,
    pub expression: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub units: String,
    pub precision: u32,
    pub fractions: String,
    #[serde(default = "default_notation")]
    pub notation: String,
    #[serde(default = "default_true", rename = "trimZeros")]
    pub trim_zeros: bool,
    #[serde(
        default,
        rename = "unitOverrides",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub unit_overrides: BTreeMap<String, String>,
    #[serde(
        default,
        rename = "customConstants",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub custom_constants: Vec<CustomConstant>,
    #[serde(
        default,
        rename = "personalDictionary",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub personal_dictionary: Vec<String>,
    #[serde(default)]
    pub page: PageSetup,
}

fn default_notation() -> String {
    "auto".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageSetup {
    pub margin_top: f64,
    pub margin_right: f64,
    pub margin_bottom: f64,
    pub margin_left: f64,
    pub header: String,
    pub footer: String,
}

impl Default for PageSetup {
    fn default() -> Self {
        Self {
            margin_top: 48.0,
            margin_right: 48.0,
            margin_bottom: 48.0,
            margin_left: 48.0,
            header: String::new(),
            footer: String::new(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            units: "metric".into(),
            precision: 4,
            fractions: "auto".into(),
            notation: "auto".into(),
            trim_zeros: true,
            unit_overrides: BTreeMap::new(),
            custom_constants: Vec::new(),
            personal_dictionary: Vec::new(),
            page: PageSetup::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub width: f64,
    pub height: f64,
    pub regions: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Region {
    Math(MathRegion),
    Text(TextRegion),
    Image(ImageRegion),
    Plot(PlotRegion),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathRegion {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub source: String,
    pub ascii: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRegion {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub src: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotSeries {
    pub expression: String,
    #[serde(default)]
    pub expression2: String,
    #[serde(default = "default_plot_kind")]
    pub kind: String,
    #[serde(default = "default_parameter_min")]
    pub parameter_min: f64,
    #[serde(default = "default_parameter_max")]
    pub parameter_max: f64,
    #[serde(default = "default_plot_color")]
    pub color: String,
    #[serde(default = "default_visible")]
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotRegion {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub series: Vec<PlotSeries>,
    pub x_min: f64,
    pub x_max: f64,
    #[serde(default)]
    pub y_min: Option<f64>,
    #[serde(default)]
    pub y_max: Option<f64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn default_plot_color() -> String {
    "#2563eb".into()
}

fn default_plot_kind() -> String {
    "explicit".into()
}

fn default_parameter_min() -> f64 {
    -10.0
}

fn default_parameter_max() -> f64 {
    10.0
}

fn default_visible() -> bool {
    true
}

impl Region {
    pub fn id(&self) -> &str {
        match self {
            Region::Math(region) => &region.id,
            Region::Text(region) => &region.id,
            Region::Image(region) => &region.id,
            Region::Plot(region) => &region.id,
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        let math_id = Uuid::new_v4().to_string();
        let text_id = Uuid::new_v4().to_string();

        Self {
            format: FORMAT_NAME.into(),
            version: FORMAT_VERSION,
            metadata: Metadata {
                title: "Sin título".into(),
                author: String::new(),
                language: "es".into(),
            },
            settings: Settings::default(),
            pages: vec![Page {
                id: Uuid::new_v4().to_string(),
                width: 850.0,
                height: 1100.0,
                regions: vec![
                    Region::Math(MathRegion {
                        id: math_id,
                        x: 60.0,
                        y: 60.0,
                        w: 360.0,
                        h: 48.0,
                        source: "2+2".into(),
                        ascii: "2+2".into(),
                        pinned: false,
                    }),
                    Region::Text(TextRegion {
                        id: text_id,
                        x: 60.0,
                        y: 140.0,
                        w: 500.0,
                        h: 100.0,
                        html: "<p>Escribe aquí...</p>".into(),
                        role: None,
                        pinned: false,
                    }),
                ],
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_pinned_region_state() {
        let mut document = Document::default();
        if let Region::Math(math) = &mut document.pages[0].regions[0] {
            math.pinned = true;
        }

        let json = serde_json::to_string(&document).expect("serialize document");
        assert!(json.contains("\"pinned\":true"));

        let loaded: Document = serde_json::from_str(&json).expect("deserialize document");
        assert!(matches!(
            &loaded.pages[0].regions[0],
            Region::Math(math) if math.pinned
        ));
    }

    #[test]
    fn preserves_number_format_settings() {
        let mut document = Document::default();
        document.settings.notation = "scientific".into();
        document.settings.precision = 6;
        document.settings.trim_zeros = false;

        let json = serde_json::to_string(&document).expect("serialize document");
        assert!(json.contains("\"trimZeros\":false"));

        let loaded: Document = serde_json::from_str(&json).expect("deserialize document");
        assert_eq!(loaded.settings.notation, "scientific");
        assert_eq!(loaded.settings.precision, 6);
        assert!(!loaded.settings.trim_zeros);
    }
}
