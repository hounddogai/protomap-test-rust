use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_assignment as plan_plan_assignment;

pub fn summarize_plan_batches_for_entry(items: &[plan_plan_assignment::PlanBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_manual_isolation_entries_from_batch(
    items: &[plan_plan_assignment::PlanBatch],
) -> Vec<ManualIsolationEntry> {
    let mut out: Vec<ManualIsolationEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_isolation_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_manual_isolation_entry(item: &ManualIsolationEntry) -> String {
    plan_plan_assignment::classify_plan_batch(item.effective_load())
}
