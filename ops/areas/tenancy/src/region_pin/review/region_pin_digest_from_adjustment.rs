use super::*;

use std::collections::BTreeMap;

use crate::migration::request as migration_request;

pub fn summarize_migration_adjustments_for_digest(
    items: &[migration_request::MigrationAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_region_pin_digests_from_adjustment(
    items: &[migration_request::MigrationAdjustment],
) -> Vec<RegionPinDigest> {
    let mut out: Vec<RegionPinDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_digest(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_region_pin_digest(item: &RegionPinDigest) -> String {
    migration_request::classify_migration_adjustment(item.effective_total())
}
