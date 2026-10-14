use super::*;

use std::collections::BTreeMap;

use crate::suspension::summary as suspension_summary;

pub fn summarize_suspension_batches_for_snapshot(
    items: &[suspension_summary::SuspensionBatch],
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

pub fn derive_plan_snapshots_from_batch(items: &[suspension_summary::SuspensionBatch]) -> Vec<PlanSnapshot> {
    let mut out: Vec<PlanSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_plan_snapshot(item: &PlanSnapshot) -> String {
    suspension_summary::classify_suspension_batch(item.effective_total())
}
