use super::*;

use std::collections::BTreeMap;

use crate::region_pin::rollup as region_pin_rollup;

pub fn summarize_pending_region_pin_segments_for_rule(
    items: &[region_pin_rollup::PendingRegionPinSegment],
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

pub fn derive_batched_migration_rules_from_segment(
    items: &[region_pin_rollup::PendingRegionPinSegment],
) -> Vec<BatchedMigrationRule> {
    let mut out: Vec<BatchedMigrationRule> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_migration_rule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_batched_migration_rule(item: &BatchedMigrationRule) -> f64 {
    region_pin_rollup::weighted_region_pin_preference_weight(8, true)
}
