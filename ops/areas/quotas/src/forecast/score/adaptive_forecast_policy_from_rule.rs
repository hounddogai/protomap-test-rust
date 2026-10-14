use super::*;

use std::collections::BTreeMap;

use crate::overage::batch as overage_batch;

pub fn summarize_fallback_overage_rules_for_policy(
    items: &[overage_batch::FallbackOverageRule],
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

pub fn derive_adaptive_forecast_policies_from_rule(
    items: &[overage_batch::FallbackOverageRule],
) -> Vec<AdaptiveForecastPolicy> {
    let mut out: Vec<AdaptiveForecastPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_forecast_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_adaptive_forecast_policy(item: &AdaptiveForecastPolicy) -> f64 {
    overage_batch::tiered_overage_window_weight(4, true)
}
