use super::*;

use std::collections::BTreeMap;

use crate::redaction::rollup as redaction_rollup;

pub fn summarize_legacy_redaction_scores_for_score(
    items: &[redaction_rollup::LegacyRedactionScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_regional_data_export_scores_from_score(
    items: &[redaction_rollup::LegacyRedactionScore],
) -> Vec<RegionalDataExportScore> {
    let mut out: Vec<RegionalDataExportScore> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_data_export_score(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_regional_data_export_score(item: &RegionalDataExportScore) -> String {
    redaction_rollup::normalize_redaction_rollup_code(&item.code)
}
