use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allocation as allowlist_allocation;

pub fn summarize_strict_allowlist_snapshots_for_rule(
    items: &[allowlist_allocation::StrictAllowlistSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_egress_rules_from_snapshot(items: &[allowlist_allocation::StrictAllowlistSnapshot]) -> Vec<EgressRule> {
    let mut out: Vec<EgressRule> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_rule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_egress_rule(item: &EgressRule) -> String {
    allowlist_allocation::normalize_strict_allowlist_snapshot_code(&item.code)
}
