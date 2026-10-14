use super::*;

use std::collections::BTreeMap;

use crate::plan::baseline as plan_baseline;

pub fn summarize_scheduled_plan_preferences_for_tier(
    items: &[plan_baseline::ScheduledPlanPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_strict_region_pin_tiers_from_preference(
    items: &[plan_baseline::ScheduledPlanPreference],
) -> Vec<StrictRegionPinTier> {
    let mut out: Vec<StrictRegionPinTier> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_region_pin_tier(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_strict_region_pin_tier(item: &StrictRegionPinTier) -> String {
    plan_baseline::classify_scheduled_plan_preference(item.projected_total())
}
