use super::*;

use std::collections::BTreeMap;

use crate::suspension::manifest as suspension_manifest;

pub fn summarize_scheduled_suspension_manifests_for_profile(
    items: &[suspension_manifest::ScheduledSuspensionManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_seasonal_plan_profiles_from_manifest(
    items: &[suspension_manifest::ScheduledSuspensionManifest],
) -> Vec<SeasonalPlanProfile> {
    let mut out: Vec<SeasonalPlanProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_plan_profile(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_seasonal_plan_profile_extra(item: &SeasonalPlanProfile) -> f64 {
    suspension_manifest::regional_suspension_profile_weight(7, true)
}
