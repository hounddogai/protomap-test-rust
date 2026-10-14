use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_milestone as forecast_forecast_milestone;

pub fn summarize_adaptive_forecast_scores_for_request(
    items: &[forecast_forecast_milestone::AdaptiveForecastScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_grant_requests_from_score(
    items: &[forecast_forecast_milestone::AdaptiveForecastScore],
) -> Vec<GrantRequest> {
    let mut out: Vec<GrantRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_request(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_grant_request(item: &GrantRequest) -> String {
    forecast_forecast_milestone::adaptive_forecast_score_bucket(item.net_total())
}
