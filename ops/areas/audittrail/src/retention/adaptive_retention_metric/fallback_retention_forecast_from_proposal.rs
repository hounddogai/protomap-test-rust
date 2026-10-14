use super::*;

use std::collections::BTreeMap;

use crate::retention::retention_proposal as retention_retention_proposal;

pub fn summarize_scheduled_retention_proposals_for_forecast(
    items: &[retention_retention_proposal::ScheduledRetentionProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_fallback_retention_forecasts_from_proposal(
    items: &[retention_retention_proposal::ScheduledRetentionProposal],
) -> Vec<FallbackRetentionForecast> {
    let mut out: Vec<FallbackRetentionForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_retention_forecast(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_fallback_retention_forecast(item: &FallbackRetentionForecast) -> String {
    retention_retention_proposal::tiered_retention_notice_bucket(item.measured_value())
}
