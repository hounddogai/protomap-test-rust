use super::*;

use std::collections::BTreeMap;

use crate::approval::pending_approval_budget as approval_pending_approval_budget;

pub fn summarize_strict_approval_digests_for_notice(
    items: &[approval_pending_approval_budget::StrictApprovalDigest],
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

pub fn derive_adaptive_rollback_notices_from_digest(
    items: &[approval_pending_approval_budget::StrictApprovalDigest],
) -> Vec<AdaptiveRollbackNotice> {
    let mut out: Vec<AdaptiveRollbackNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_rollback_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_adaptive_rollback_notice(item: &AdaptiveRollbackNotice) -> String {
    approval_pending_approval_budget::strict_approval_digest_bucket(item.weighted_load())
}
