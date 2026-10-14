use super::*;

use std::collections::BTreeMap;

use crate::rollback::rollback_policy as rollback_rollback_policy;

pub fn summarize_manual_rollback_digests_for_mapping(
    items: &[rollback_rollback_policy::ManualRollbackDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_approval_mappings_from_digest(
    items: &[rollback_rollback_policy::ManualRollbackDigest],
) -> Vec<ApprovalMapping> {
    let mut out: Vec<ApprovalMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_mapping(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_approval_mapping(item: &ApprovalMapping) -> String {
    rollback_rollback_policy::legacy_rollback_policy_bucket(item.weighted_exposure())
}
