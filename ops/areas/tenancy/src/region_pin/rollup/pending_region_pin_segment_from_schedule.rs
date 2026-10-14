use super::*;

use std::collections::BTreeMap;

use crate::migration::proposal as migration_proposal;

pub fn summarize_legacy_migration_schedules_for_segment(
    items: &[migration_proposal::LegacyMigrationSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_pending_region_pin_segments_from_schedule(
    items: &[migration_proposal::LegacyMigrationSchedule],
) -> Vec<PendingRegionPinSegment> {
    let mut out: Vec<PendingRegionPinSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_region_pin_segment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_pending_region_pin_segment(item: &PendingRegionPinSegment) -> f64 {
    migration_proposal::manual_migration_estimate_weight(6, true)
}
