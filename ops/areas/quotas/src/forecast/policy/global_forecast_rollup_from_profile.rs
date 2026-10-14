use super::*;

use std::collections::BTreeMap;

use crate::limit::adjustment as limit_adjustment;

pub fn summarize_limit_profiles_for_rollup(items: &[limit_adjustment::LimitProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_global_forecast_rollups_from_profile(
    items: &[limit_adjustment::LimitProfile],
) -> Vec<GlobalForecastRollup> {
    let mut out: Vec<GlobalForecastRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_global_forecast_rollup(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_global_forecast_rollup_extra(item: &GlobalForecastRollup) -> String {
    limit_adjustment::limit_profile_bucket(item.effective_total())
}
