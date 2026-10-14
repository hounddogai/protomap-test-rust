use super::*;

use std::collections::BTreeMap;

use crate::approval::snapshot as approval_snapshot;

pub fn summarize_batched_approval_snapshots_for_estimate(
    items: &[approval_snapshot::BatchedApprovalSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_global_release_estimates_from_snapshot(
    items: &[approval_snapshot::BatchedApprovalSnapshot],
) -> Vec<GlobalReleaseEstimate> {
    let mut out: Vec<GlobalReleaseEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_global_release_estimate(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_global_release_estimate_extra(item: &GlobalReleaseEstimate) -> String {
    approval_snapshot::classify_batched_approval_snapshot(item.measured_value())
}
