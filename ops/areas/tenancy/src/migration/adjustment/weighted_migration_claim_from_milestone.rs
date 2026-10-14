use super::*;

use std::collections::BTreeMap;

use crate::region_pin::metric as region_pin_metric;

pub fn summarize_capped_region_pin_milestones_for_claim(
    items: &[region_pin_metric::CappedRegionPinMilestone],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_weighted_migration_claims_from_milestone(
    items: &[region_pin_metric::CappedRegionPinMilestone],
) -> Vec<WeightedMigrationClaim> {
    let mut out: Vec<WeightedMigrationClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_migration_claim(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_weighted_migration_claim(item: &WeightedMigrationClaim) -> String {
    region_pin_metric::normalize_region_pin_schedule_code(&item.code)
}
