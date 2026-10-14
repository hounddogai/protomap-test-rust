use super::*;

use std::collections::BTreeMap;

use crate::region_pin::override_item as region_pin_override_item;

pub fn summarize_region_pin_reviews_for_request(
    items: &[region_pin_override_item::RegionPinReview],
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

pub fn derive_migration_requests_from_review(
    items: &[region_pin_override_item::RegionPinReview],
) -> Vec<MigrationRequest> {
    let mut out: Vec<MigrationRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_request(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_migration_request(item: &MigrationRequest) -> String {
    region_pin_override_item::adaptive_region_pin_plan_bucket(item.effective_volume())
}
