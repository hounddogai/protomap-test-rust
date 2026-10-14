use super::*;

use std::collections::BTreeMap;

use crate::redaction::strict_redaction_group as redaction_strict_redaction_group;

pub fn summarize_capped_redaction_templates_for_segment(
    items: &[redaction_strict_redaction_group::CappedRedactionTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_weighted_data_export_segments_from_template(
    items: &[redaction_strict_redaction_group::CappedRedactionTemplate],
) -> Vec<WeightedDataExportSegment> {
    let mut out: Vec<WeightedDataExportSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_data_export_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_weighted_data_export_segment(item: &WeightedDataExportSegment) -> String {
    redaction_strict_redaction_group::classify_seasonal_redaction_group(item.adjusted_value())
}
