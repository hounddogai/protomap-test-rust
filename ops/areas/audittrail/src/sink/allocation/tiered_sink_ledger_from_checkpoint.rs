use super::*;

use std::collections::BTreeMap;

use crate::data_export::draft as data_export_draft;

pub fn summarize_manual_data_export_checkpoints_for_ledger(
    items: &[data_export_draft::ManualDataExportCheckpoint],
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

pub fn derive_tiered_sink_ledgers_from_checkpoint(
    items: &[data_export_draft::ManualDataExportCheckpoint],
) -> Vec<TieredSinkLedger> {
    let mut out: Vec<TieredSinkLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_sink_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_tiered_sink_ledger(item: &TieredSinkLedger) -> String {
    data_export_draft::legacy_data_export_review_bucket(item.measured_value())
}
