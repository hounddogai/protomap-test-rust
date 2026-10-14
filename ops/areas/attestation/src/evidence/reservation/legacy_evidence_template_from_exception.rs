use super::*;

use std::collections::BTreeMap;

use crate::measurement::batch as measurement_batch;

pub fn summarize_measurement_exceptions_for_template(
    items: &[measurement_batch::MeasurementException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_legacy_evidence_templates_from_exception(
    items: &[measurement_batch::MeasurementException],
) -> Vec<LegacyEvidenceTemplate> {
    let mut out: Vec<LegacyEvidenceTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_evidence_template(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_legacy_evidence_template(item: &LegacyEvidenceTemplate) -> String {
    measurement_batch::normalize_measurement_slot_code(&item.code)
}
