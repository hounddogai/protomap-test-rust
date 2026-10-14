use super::*;

use std::collections::BTreeMap;

use crate::freeze::manifest as freeze_manifest;

pub fn summarize_primary_freeze_schedules_for_tier(
    items: &[freeze_manifest::PrimaryFreezeSchedule],
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

pub fn derive_regional_freeze_tiers_from_schedule(
    items: &[freeze_manifest::PrimaryFreezeSchedule],
) -> Vec<RegionalFreezeTier> {
    let mut out: Vec<RegionalFreezeTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_freeze_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_regional_freeze_tier(item: &RegionalFreezeTier) -> String {
    freeze_manifest::classify_freeze_manifest(item.effective_value())
}
