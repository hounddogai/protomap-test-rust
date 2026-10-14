use super::*;

use std::collections::BTreeMap;

use crate::grant::snapshot as grant_snapshot;

pub fn summarize_grant_policies_for_batch(items: &[grant_snapshot::GrantPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_pending_overage_batches_from_policy(items: &[grant_snapshot::GrantPolicy]) -> Vec<PendingOverageBatch> {
    let mut out: Vec<PendingOverageBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_overage_batch(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_overage_batch(item: &PendingOverageBatch) -> f64 {
    grant_snapshot::grant_adjustment_weight(9, true)
}
