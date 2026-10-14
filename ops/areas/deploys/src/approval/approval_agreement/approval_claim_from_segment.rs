use super::*;

use std::collections::BTreeMap;

use crate::approval::snapshot as approval_snapshot;

pub fn summarize_approval_segments_for_claim(items: &[approval_snapshot::ApprovalSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_approval_claims_from_segment(items: &[approval_snapshot::ApprovalSegment]) -> Vec<ApprovalClaim> {
    let mut out: Vec<ApprovalClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_approval_claim(item: &ApprovalClaim) -> String {
    approval_snapshot::fallback_approval_summary_bucket(item.net_volume())
}
