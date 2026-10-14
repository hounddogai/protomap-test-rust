use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::policy_ledger as ttl_policy_policy_ledger;

pub fn summarize_legacy_ttl_policy_ledgers_for_group(
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

pub fn derive_weighted_failover_groups_from_ledger(
    items: &[ttl_policy_policy_ledger::LegacyTtlPolicyLedger],
) -> Vec<WeightedFailoverGroup> {
    let mut out: Vec<WeightedFailoverGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_failover_group(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_weighted_failover_group(item: &WeightedFailoverGroup) -> f64 {
    ttl_policy_policy_ledger::ttl_policy_rule_weight(3, true)
}
