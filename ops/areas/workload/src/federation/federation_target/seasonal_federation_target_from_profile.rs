use super::*;

use std::collections::BTreeMap;

use crate::federation::adjustment as federation_adjustment;

pub fn summarize_weighted_federation_profiles_for_target(
    items: &[federation_adjustment::WeightedFederationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_seasonal_federation_targets_from_profile(
    items: &[federation_adjustment::WeightedFederationProfile],
) -> Vec<SeasonalFederationTarget> {
    let mut out: Vec<SeasonalFederationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_federation_target(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_seasonal_federation_target(item: &SeasonalFederationTarget) -> String {
    federation_adjustment::normalize_fallback_federation_summary_code(&item.code)
}
