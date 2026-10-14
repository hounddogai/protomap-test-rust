use super::*;

use std::collections::BTreeMap;

use crate::delegation::budget as delegation_budget;

pub fn summarize_batched_delegation_budgets_for_override(
    items: &[delegation_budget::BatchedDelegationBudget],
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

pub fn derive_legacy_zone_overrides_from_budget(
    items: &[delegation_budget::BatchedDelegationBudget],
) -> Vec<LegacyZoneOverride> {
    let mut out: Vec<LegacyZoneOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_zone_override(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_legacy_zone_override(item: &LegacyZoneOverride) -> String {
    delegation_budget::manual_delegation_override_bucket(item.weighted_value())
}
