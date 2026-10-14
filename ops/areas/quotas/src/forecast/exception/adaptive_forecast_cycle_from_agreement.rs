use super::*;

use std::collections::BTreeMap;

use crate::limit::mapping as limit_mapping;

pub fn summarize_limit_agreements_for_cycle(items: &[limit_mapping::LimitAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_adaptive_forecast_cycles_from_agreement(
    items: &[limit_mapping::LimitAgreement],
) -> Vec<AdaptiveForecastCycle> {
    let mut out: Vec<AdaptiveForecastCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_forecast_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_adaptive_forecast_cycle(item: &AdaptiveForecastCycle) -> f64 {
    limit_mapping::limit_override_weight(9, true)
}
