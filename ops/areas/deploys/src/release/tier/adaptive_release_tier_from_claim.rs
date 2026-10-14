use super::*;

use std::collections::BTreeMap;

use crate::release::strict_release_policy as release_strict_release_policy;

pub fn summarize_primary_release_claims_for_tier(
    items: &[release_strict_release_policy::PrimaryReleaseClaim],
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

pub fn derive_adaptive_release_tiers_from_claim(
    items: &[release_strict_release_policy::PrimaryReleaseClaim],
) -> Vec<AdaptiveReleaseTier> {
    let mut out: Vec<AdaptiveReleaseTier> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_release_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_adaptive_release_tier(item: &AdaptiveReleaseTier) -> String {
    release_strict_release_policy::normalize_primary_release_claim_code(&item.code)
}
