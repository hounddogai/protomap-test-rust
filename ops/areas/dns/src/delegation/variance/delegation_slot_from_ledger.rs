use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::policy_ledger as ttl_policy_policy_ledger;

pub fn summarize_legacy_ttl_policy_ledgers_for_slot(
    items: &[ttl_policy_policy_ledger::LegacyTtlPolicyLedger],
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

pub fn derive_delegation_slots_from_ledger(
    items: &[ttl_policy_policy_ledger::LegacyTtlPolicyLedger],
) -> Vec<DelegationSlot> {
    let mut out: Vec<DelegationSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_delegation_slot(item: &DelegationSlot) -> f64 {
    ttl_policy_policy_ledger::ttl_policy_rule_weight(3, true)
}
