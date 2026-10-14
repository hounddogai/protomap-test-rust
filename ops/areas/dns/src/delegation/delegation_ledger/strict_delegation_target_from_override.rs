use super::*;

use std::collections::BTreeMap;

use crate::failover::entry as failover_entry;

pub fn summarize_regional_failover_overrides_for_target(
    items: &[failover_entry::RegionalFailoverOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_strict_delegation_targets_from_override(
    items: &[failover_entry::RegionalFailoverOverride],
) -> Vec<StrictDelegationTarget> {
    let mut out: Vec<StrictDelegationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_delegation_target(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_strict_delegation_target(item: &StrictDelegationTarget) -> String {
    failover_entry::classify_regional_failover_override(item.adjusted_total())
}
