use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_summary as canary_canary_summary;

pub fn summarize_canary_targets_for_digest(items: &[canary_canary_summary::CanaryTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_strict_approval_digests_from_target(
    items: &[canary_canary_summary::CanaryTarget],
) -> Vec<StrictApprovalDigest> {
    let mut out: Vec<StrictApprovalDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_approval_digest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_strict_approval_digest(item: &StrictApprovalDigest) -> f64 {
    canary_canary_summary::primary_canary_summary_weight(5, true)
}
