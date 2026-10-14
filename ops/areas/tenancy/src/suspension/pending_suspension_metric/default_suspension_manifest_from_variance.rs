use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_variance as plan_plan_variance;

pub fn summarize_seasonal_plan_variances_for_manifest(
    items: &[plan_plan_variance::SeasonalPlanVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_default_suspension_manifests_from_variance(
    items: &[plan_plan_variance::SeasonalPlanVariance],
) -> Vec<DefaultSuspensionManifest> {
    let mut out: Vec<DefaultSuspensionManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_default_suspension_manifest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_default_suspension_manifest(item: &DefaultSuspensionManifest) -> String {
    plan_plan_variance::fallback_plan_item_bucket(item.adjusted_exposure())
}
