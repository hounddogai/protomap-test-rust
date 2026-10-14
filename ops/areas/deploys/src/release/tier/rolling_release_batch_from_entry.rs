use super::*;

use std::collections::BTreeMap;

use crate::approval::score as approval_score;

pub fn summarize_strict_approval_entries_for_batch(
    items: &[approval_score::StrictApprovalEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_rolling_release_batches_from_entry(
    items: &[approval_score::StrictApprovalEntry],
) -> Vec<RollingReleaseBatch> {
    let mut out: Vec<RollingReleaseBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_release_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_rolling_release_batch(item: &RollingReleaseBatch) -> String {
    approval_score::classify_strict_approval_entry(item.adjusted_load())
}
