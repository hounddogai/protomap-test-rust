use super::*;

use std::collections::BTreeMap;

use crate::failover::group as failover_group;

pub fn summarize_primary_failover_groups_for_ledger(
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

pub fn derive_ttl_policy_ledgers_from_group(items: &[failover_group::PrimaryFailoverGroup]) -> Vec<TtlPolicyLedger> {
    let mut out: Vec<TtlPolicyLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_ledger(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_ttl_policy_ledger(item: &TtlPolicyLedger) -> String {
    failover_group::classify_failover_exception(item.projected_volume())
}
