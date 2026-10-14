use super::*;

use std::collections::BTreeMap;

use crate::canary::reservation as canary_reservation;

pub fn summarize_capped_canary_rules_for_preference(
    items: &[canary_reservation::CappedCanaryRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_tiered_approval_preferences_from_rule(
    items: &[canary_reservation::CappedCanaryRule],
) -> Vec<TieredApprovalPreference> {
    let mut out: Vec<TieredApprovalPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_approval_preference(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_tiered_approval_preference(item: &TieredApprovalPreference) -> f64 {
    canary_reservation::capped_canary_rule_weight(1, true)
}
