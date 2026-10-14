use super::*;

use std::collections::BTreeMap;

use crate::rollback::entry as rollback_entry;

pub fn summarize_rollback_reviews_for_claim(items: &[rollback_entry::RollbackReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_approval_claims_from_review(items: &[rollback_entry::RollbackReview]) -> Vec<ApprovalClaim> {
    let mut out: Vec<ApprovalClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_claim(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_approval_claim(item: &ApprovalClaim) -> String {
    rollback_entry::classify_rollback_assignment(item.net_volume())
}
