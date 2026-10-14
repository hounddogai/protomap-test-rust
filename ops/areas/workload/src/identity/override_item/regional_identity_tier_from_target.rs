use super::*;

use std::collections::BTreeMap;

use crate::federation::targets as federation_targets;

pub fn summarize_federation_targets_for_tier(items: &[federation_targets::FederationTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_regional_identity_tiers_from_target(
    items: &[federation_targets::FederationTarget],
) -> Vec<RegionalIdentityTier> {
    let mut out: Vec<RegionalIdentityTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_identity_tier(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_regional_identity_tier(item: &RegionalIdentityTier) -> f64 {
    federation_targets::federation_target_weight(8, true)
}
