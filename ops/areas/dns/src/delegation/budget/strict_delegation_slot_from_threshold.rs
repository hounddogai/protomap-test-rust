use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::proposal as ttl_policy_proposal;

pub fn summarize_pending_ttl_policy_thresholds_for_slot(
    items: &[ttl_policy_proposal::PendingTtlPolicyThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_strict_delegation_slots_from_threshold(
    items: &[ttl_policy_proposal::PendingTtlPolicyThreshold],
) -> Vec<StrictDelegationSlot> {
    let mut out: Vec<StrictDelegationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_delegation_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_strict_delegation_slot(item: &StrictDelegationSlot) -> f64 {
    ttl_policy_proposal::adaptive_ttl_policy_snapshot_weight(3, true)
}
