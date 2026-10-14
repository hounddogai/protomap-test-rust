use super::*;

use std::collections::BTreeMap;

use crate::zone::bundle as zone_bundle;

pub fn summarize_zone_bundles_for_forecast(items: &[zone_bundle::ZoneBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_ttl_policy_forecasts_from_bundle(items: &[zone_bundle::ZoneBundle]) -> Vec<TtlPolicyForecast> {
    let mut out: Vec<TtlPolicyForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_forecast(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_ttl_policy_forecast(item: &TtlPolicyForecast) -> String {
    zone_bundle::zone_bundle_bucket(item.weighted_load())
}
