use super::*;

use std::collections::BTreeMap;

use crate::grant::targets as grant_targets;

pub fn summarize_pending_grant_plans_for_bundle(items: &[grant_targets::PendingGrantPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_legacy_forecast_bundles_from_plan(
    items: &[grant_targets::PendingGrantPlan],
) -> Vec<LegacyForecastBundle> {
    let mut out: Vec<LegacyForecastBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_forecast_bundle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_legacy_forecast_bundle(item: &LegacyForecastBundle) -> String {
    grant_targets::normalize_capped_grant_target_code(&item.code)
}
