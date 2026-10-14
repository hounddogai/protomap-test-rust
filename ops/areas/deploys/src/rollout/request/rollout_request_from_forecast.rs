use super::*;

use std::collections::BTreeMap;

use crate::rollout::notice as rollout_notice;

pub fn summarize_legacy_rollout_forecasts_for_request(
    items: &[rollout_notice::LegacyRolloutForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_rollout_requests_from_forecast(items: &[rollout_notice::LegacyRolloutForecast]) -> Vec<RolloutRequest> {
    let mut out: Vec<RolloutRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_request(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_rollout_request(item: &RolloutRequest) -> String {
    rollout_notice::normalize_legacy_rollout_forecast_code(&item.code)
}
