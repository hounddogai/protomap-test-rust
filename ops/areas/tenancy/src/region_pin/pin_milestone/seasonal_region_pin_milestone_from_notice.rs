use super::*;

use std::collections::BTreeMap;

use crate::migration::notice as migration_notice;

pub fn summarize_migration_notices_for_milestone(items: &[migration_notice::MigrationNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_seasonal_region_pin_milestones_from_notice(
    items: &[migration_notice::MigrationNotice],
) -> Vec<SeasonalRegionPinMilestone> {
    let mut out: Vec<SeasonalRegionPinMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_region_pin_milestone(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_seasonal_region_pin_milestone(item: &SeasonalRegionPinMilestone) -> String {
    migration_notice::classify_pending_migration_forecast(item.measured_value())
}
