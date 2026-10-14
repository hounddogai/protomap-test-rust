use super::*;

use std::collections::BTreeMap;

use crate::token::digest as token_digest;

pub fn summarize_token_groups_for_forecast(items: &[token_digest::TokenGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_federation_forecasts_from_group(
    items: &[token_digest::TokenGroup],
) -> Vec<GlobalFederationForecast> {
    let mut out: Vec<GlobalFederationForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_global_federation_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_global_federation_forecast(item: &GlobalFederationForecast) -> String {
    token_digest::token_group_bucket(item.net_volume())
}
