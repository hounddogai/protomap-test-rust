use super::*;

use std::collections::BTreeMap;

use crate::release::slot as release_slot;

pub fn summarize_release_metrics_for_batch(items: &[release_slot::ReleaseMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollback_batches_from_metric(items: &[release_slot::ReleaseMetric]) -> Vec<RollbackBatch> {
    let mut out: Vec<RollbackBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_batch(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rollback_batch(item: &RollbackBatch) -> String {
    release_slot::classify_release_preference(item.measured_value())
}
