use super::*;

use std::collections::BTreeMap;

use crate::federation::metric as federation_metric;

pub fn summarize_tiered_federation_groups_for_profile(
    items: &[federation_metric::TieredFederationGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_weighted_federation_profiles_from_group(
    items: &[federation_metric::TieredFederationGroup],
) -> Vec<WeightedFederationProfile> {
    let mut out: Vec<WeightedFederationProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_federation_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_weighted_federation_profile(item: &WeightedFederationProfile) -> String {
    federation_metric::normalize_primary_federation_preference_code(&item.code)
}
