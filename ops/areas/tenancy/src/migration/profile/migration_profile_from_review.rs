use super::*;

use std::collections::BTreeMap;

use crate::region_pin::review as region_pin_review;

pub fn summarize_adaptive_region_pin_reviews_for_profile(
    items: &[region_pin_review::AdaptiveRegionPinReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_migration_profiles_from_review(
    items: &[region_pin_review::AdaptiveRegionPinReview],
) -> Vec<MigrationProfile> {
    let mut out: Vec<MigrationProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_migration_profile(item: &MigrationProfile) -> String {
    region_pin_review::classify_adaptive_region_pin_review(item.weighted_volume())
}
