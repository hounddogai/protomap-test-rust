use super::*;

use std::collections::BTreeMap;

use crate::sink::assignment as sink_assignment;

pub fn summarize_lenient_sink_plans_for_batch(items: &[sink_assignment::LenientSinkPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_legacy_data_export_batches_from_plan(
    items: &[sink_assignment::LenientSinkPlan],
) -> Vec<LegacyDataExportBatch> {
    let mut out: Vec<LegacyDataExportBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_data_export_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_legacy_data_export_batch(item: &LegacyDataExportBatch) -> f64 {
    sink_assignment::lenient_sink_plan_weight(6, true)
}
