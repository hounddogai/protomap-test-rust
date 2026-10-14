use super::*;

use std::collections::BTreeMap;

use crate::redaction::score as redaction_score;

pub fn summarize_pending_redaction_adjustments_for_batch(
    items: &[redaction_score::PendingRedactionAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_capped_data_export_batches_from_adjustment(
    items: &[redaction_score::PendingRedactionAdjustment],
) -> Vec<CappedDataExportBatch> {
    let mut out: Vec<CappedDataExportBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_data_export_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_capped_data_export_batch(item: &CappedDataExportBatch) -> String {
    redaction_score::normalize_legacy_redaction_rollup_code(&item.code)
}
