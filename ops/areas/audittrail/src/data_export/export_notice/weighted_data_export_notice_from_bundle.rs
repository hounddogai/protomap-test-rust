use super::*;

use std::collections::BTreeMap;

use crate::redaction::entry as redaction_entry;

pub fn summarize_batched_redaction_bundles_for_notice(
    items: &[redaction_entry::BatchedRedactionBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_weighted_data_export_notices_from_bundle(
    items: &[redaction_entry::BatchedRedactionBundle],
) -> Vec<WeightedDataExportNotice> {
    let mut out: Vec<WeightedDataExportNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_data_export_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_weighted_data_export_notice(item: &WeightedDataExportNotice) -> String {
    redaction_entry::normalize_default_redaction_entry_code(&item.code)
}
