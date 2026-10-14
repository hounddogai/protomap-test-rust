use super::*;

use std::collections::BTreeMap;

use crate::federation::allocation as federation_allocation;

pub fn summarize_legacy_federation_adjustments_for_profile(
    items: &[federation_allocation::LegacyFederationAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_seasonal_identity_profiles_from_adjustment(
    items: &[federation_allocation::LegacyFederationAdjustment],
) -> Vec<SeasonalIdentityProfile> {
    let mut out: Vec<SeasonalIdentityProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_identity_profile(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_seasonal_identity_profile(item: &SeasonalIdentityProfile) -> String {
    federation_allocation::pending_federation_allocation_bucket(item.adjusted_total())
}
