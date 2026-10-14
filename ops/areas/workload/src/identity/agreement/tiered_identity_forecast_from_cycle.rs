use super::*;

use std::collections::BTreeMap;

use crate::identity::cycle as identity_cycle;

pub fn summarize_identity_cycles_for_forecast(items: &[identity_cycle::IdentityCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_identity_forecasts_from_cycle(
    items: &[identity_cycle::IdentityCycle],
) -> Vec<TieredIdentityForecast> {
    let mut out: Vec<TieredIdentityForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_identity_forecast(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_tiered_identity_forecast(item: &TieredIdentityForecast) -> String {
    identity_cycle::identity_cycle_bucket(item.measured_value())
}
