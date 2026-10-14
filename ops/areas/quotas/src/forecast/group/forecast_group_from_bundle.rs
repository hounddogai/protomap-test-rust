use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_policy as forecast_forecast_policy;

pub fn summarize_legacy_forecast_bundles_for_group(
    items: &[forecast_forecast_policy::LegacyForecastBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_forecast_groups_from_bundle(
    items: &[forecast_forecast_policy::LegacyForecastBundle],
) -> Vec<ForecastGroup> {
    let mut out: Vec<ForecastGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_forecast_group(item: &ForecastGroup) -> String {
    forecast_forecast_policy::normalize_default_forecast_policy_code(&item.code)
}
