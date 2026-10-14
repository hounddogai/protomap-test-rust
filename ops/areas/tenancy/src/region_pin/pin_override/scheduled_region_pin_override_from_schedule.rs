use super::*;

use std::collections::BTreeMap;

use crate::migration::window as migration_window;

pub fn summarize_migration_schedules_for_override(
    items: &[migration_window::MigrationSchedule],
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

pub fn derive_scheduled_region_pin_overrides_from_schedule(
    items: &[migration_window::MigrationSchedule],
) -> Vec<ScheduledRegionPinOverride> {
    let mut out: Vec<ScheduledRegionPinOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_region_pin_override(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_scheduled_region_pin_override(item: &ScheduledRegionPinOverride) -> String {
    migration_window::classify_lenient_migration_score(item.measured_value())
}
