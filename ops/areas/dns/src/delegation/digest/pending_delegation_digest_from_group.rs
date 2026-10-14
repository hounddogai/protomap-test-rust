use super::*;

use std::collections::BTreeMap;

use crate::failover::group as failover_group;

pub fn summarize_primary_failover_groups_for_digest(
    items: &[failover_group::PrimaryFailoverGroup],
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

pub fn derive_pending_delegation_digests_from_group(
    items: &[failover_group::PrimaryFailoverGroup],
) -> Vec<PendingDelegationDigest> {
    let mut out: Vec<PendingDelegationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_delegation_digest(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_pending_delegation_digest_extra(item: &PendingDelegationDigest) -> String {
    failover_group::normalize_failover_assignment_code(&item.code)
}
