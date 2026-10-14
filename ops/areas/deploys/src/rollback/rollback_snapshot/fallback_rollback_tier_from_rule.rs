use super::*;

use std::collections::BTreeMap;

use crate::release::allocation as release_allocation;

pub fn summarize_default_release_rules_for_tier(
    items: &[release_allocation::DefaultReleaseRule],
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

pub fn derive_fallback_rollback_tiers_from_rule(
    items: &[release_allocation::DefaultReleaseRule],
) -> Vec<FallbackRollbackTier> {
    let mut out: Vec<FallbackRollbackTier> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_rollback_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_fallback_rollback_tier(item: &FallbackRollbackTier) -> String {
    release_allocation::classify_global_release_allocation(item.adjusted_exposure())
}
