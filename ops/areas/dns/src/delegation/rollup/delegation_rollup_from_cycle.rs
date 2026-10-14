use super::*;

use std::collections::BTreeMap;

use crate::failover::request as failover_request;

pub fn summarize_failover_cycles_for_rollup(items: &[failover_request::FailoverCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_delegation_rollups_from_cycle(items: &[failover_request::FailoverCycle]) -> Vec<DelegationRollup> {
    let mut out: Vec<DelegationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_delegation_rollup(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_delegation_rollup(item: &DelegationRollup) -> String {
    failover_request::failover_cycle_bucket(item.weighted_volume())
}
