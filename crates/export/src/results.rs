// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use doc_core::{Document, Region};

/// Evaluate every math region once and keep the display text of successful
/// results keyed by region id. Errors and empty regions are skipped.
pub(crate) fn evaluate_math_results(document: &Document) -> HashMap<String, String> {
    let items = document
        .pages
        .iter()
        .flat_map(|page| page.regions.iter())
        .filter_map(|region| match region {
            Region::Math(math) if !math.ascii.trim().is_empty() => {
                Some((math.id.clone(), math.ascii.clone()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let constants = document
        .settings
        .custom_constants
        .iter()
        .map(|constant| (constant.name.clone(), constant.expression.clone()))
        .collect::<Vec<_>>();

    math_eval::evaluate_document_with_constants_and_preferences(
        &items,
        &constants,
        &math_eval::UnitPreferences::new(
            document.settings.units.clone(),
            document.settings.unit_overrides.clone(),
        ),
    )
    .into_iter()
    .filter_map(|evaluation| match evaluation.result {
        Ok(value) => Some((
            evaluation.id,
            value.display_with_format(&math_eval::NumberFormat::from_settings(
                &document.settings.notation,
                document.settings.precision,
                document.settings.trim_zeros,
                &document.settings.fractions,
            )),
        )),
        Err(_) => None,
    })
    .collect()
}
