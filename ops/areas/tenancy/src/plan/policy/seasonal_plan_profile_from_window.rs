use super::*;

use std::collections::BTreeMap;

use crate::region_pin::legacy_pin_override as region_pin_legacy_pin_override;

pub fn summarize_region_pin_windows_for_profile(
    items: &[region_pin_legacy_pin_override::RegionPinWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_seasonal_plan_profiles_from_window(
    items: &[region_pin_legacy_pin_override::RegionPinWindow],
) -> Vec<SeasonalPlanProfile> {
    let mut out: Vec<SeasonalPlanProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_plan_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_seasonal_plan_profile(item: &SeasonalPlanProfile) -> String {
    region_pin_legacy_pin_override::region_pin_budget_bucket(item.projected_total())
}
