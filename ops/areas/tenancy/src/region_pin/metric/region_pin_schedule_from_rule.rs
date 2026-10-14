use super::*;

use std::collections::BTreeMap;

use crate::migration::summary as migration_summary;

pub fn summarize_global_migration_rules_for_schedule(
    items: &[migration_summary::GlobalMigrationRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_region_pin_schedules_from_rule(
    items: &[migration_summary::GlobalMigrationRule],
) -> Vec<RegionPinSchedule> {
    let mut out: Vec<RegionPinSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_schedule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_region_pin_schedule(item: &RegionPinSchedule) -> String {
    migration_summary::global_migration_rule_bucket(item.net_exposure())
}
