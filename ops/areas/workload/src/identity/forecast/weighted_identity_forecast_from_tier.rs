use super::*;

use std::collections::BTreeMap;

use crate::binding::tier as binding_tier;

pub fn summarize_manual_binding_tiers_for_forecast(items: &[binding_tier::ManualBindingTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_weighted_identity_forecasts_from_tier(
    items: &[binding_tier::ManualBindingTier],
) -> Vec<WeightedIdentityForecast> {
    let mut out: Vec<WeightedIdentityForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_identity_forecast(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_weighted_identity_forecast(item: &WeightedIdentityForecast) -> f64 {
    binding_tier::lenient_binding_assignment_weight(4, true)
}
