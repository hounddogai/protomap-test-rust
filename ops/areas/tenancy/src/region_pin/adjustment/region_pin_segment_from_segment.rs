use super::*;

use std::collections::BTreeMap;

use crate::migration::migration_segment as migration_migration_segment;

pub fn summarize_scheduled_migration_segments_for_segment(
    items: &[migration_migration_segment::ScheduledMigrationSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_region_pin_segments_from_segment(
    items: &[migration_migration_segment::ScheduledMigrationSegment],
) -> Vec<RegionPinSegment> {
    let mut out: Vec<RegionPinSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_region_pin_segment(item: &RegionPinSegment) -> String {
    migration_migration_segment::scheduled_migration_segment_bucket(item.weighted_volume())
}
