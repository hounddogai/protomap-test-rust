use super::*;

use std::collections::BTreeMap;

use crate::canary::rollup as canary_rollup;

pub fn summarize_pending_canary_mappings_for_digest(
    items: &[canary_rollup::PendingCanaryMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_pending_approval_digests_from_mapping(
    items: &[canary_rollup::PendingCanaryMapping],
) -> Vec<PendingApprovalDigest> {
    let mut out: Vec<PendingApprovalDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_approval_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_pending_approval_digest(item: &PendingApprovalDigest) -> String {
    canary_rollup::classify_pending_canary_mapping(item.projected_total())
}
