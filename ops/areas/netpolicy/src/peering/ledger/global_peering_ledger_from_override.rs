use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allowlist_override as allowlist_allowlist_override;

pub fn summarize_strict_allowlist_overrides_for_ledger(
    items: &[allowlist_allowlist_override::StrictAllowlistOverride],
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

pub fn derive_global_peering_ledgers_from_override(
    items: &[allowlist_allowlist_override::StrictAllowlistOverride],
) -> Vec<GlobalPeeringLedger> {
    let mut out: Vec<GlobalPeeringLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_global_peering_ledger(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_global_peering_ledger(item: &GlobalPeeringLedger) -> String {
    allowlist_allowlist_override::adaptive_allowlist_budget_bucket(item.measured_value())
}
