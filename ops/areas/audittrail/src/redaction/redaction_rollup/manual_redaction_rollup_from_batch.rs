use super::*;

use std::collections::BTreeMap;

use crate::data_export::batch as data_export_batch;

pub fn summarize_data_export_batches_for_rollup(items: &[data_export_batch::DataExportBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_manual_redaction_rollups_from_batch(
    items: &[data_export_batch::DataExportBatch],
) -> Vec<ManualRedactionRollup> {
    let mut out: Vec<ManualRedactionRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_redaction_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_manual_redaction_rollup(item: &ManualRedactionRollup) -> f64 {
    data_export_batch::data_export_batch_weight(3, true)
}
