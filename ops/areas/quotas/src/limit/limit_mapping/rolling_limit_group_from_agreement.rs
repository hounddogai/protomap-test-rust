use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_estimate as forecast_forecast_estimate;

pub fn summarize_tiered_forecast_agreements_for_group(
    items: &[forecast_forecast_estimate::TieredForecastAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_rolling_limit_groups_from_agreement(
    items: &[forecast_forecast_estimate::TieredForecastAgreement],
) -> Vec<RollingLimitGroup> {
    let mut out: Vec<RollingLimitGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_limit_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_rolling_limit_group(item: &RollingLimitGroup) -> String {
    forecast_forecast_estimate::normalize_batched_forecast_reservation_code(&item.code)
}
