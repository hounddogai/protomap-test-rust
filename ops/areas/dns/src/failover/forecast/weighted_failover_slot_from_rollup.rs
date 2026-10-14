use super::*;

use std::collections::BTreeMap;

use crate::delegation::rollup as delegation_rollup;

pub fn summarize_delegation_rollups_for_slot(items: &[delegation_rollup::DelegationRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_weighted_failover_slots_from_rollup(
    items: &[delegation_rollup::DelegationRollup],
) -> Vec<WeightedFailoverSlot> {
    let mut out: Vec<WeightedFailoverSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_failover_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_weighted_failover_slot(item: &WeightedFailoverSlot) -> String {
    delegation_rollup::classify_delegation_forecast(item.net_total())
}
