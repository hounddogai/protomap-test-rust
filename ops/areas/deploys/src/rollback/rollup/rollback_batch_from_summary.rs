use super::*;

use std::collections::BTreeMap;

use crate::canary::checkpoint as canary_checkpoint;

pub fn summarize_canary_summaries_for_batch(items: &[canary_checkpoint::CanarySummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_rollback_batches_from_summary(items: &[canary_checkpoint::CanarySummary]) -> Vec<RollbackBatch> {
    let mut out: Vec<RollbackBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_batch(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollback_batch(item: &RollbackBatch) -> String {
    canary_checkpoint::classify_canary_checkpoint(item.measured_value())
}
