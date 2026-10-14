use super::*;

use std::collections::BTreeMap;

use crate::forecast::quota as forecast_quota;

pub fn summarize_forecast_agreements_for_proposal(
    items: &[forecast_quota::ForecastAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_usage_proposals_from_agreement(items: &[forecast_quota::ForecastAgreement]) -> Vec<UsageProposal> {
    let mut out: Vec<UsageProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_proposal(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_usage_proposal(item: &UsageProposal) -> String {
    forecast_quota::forecast_quota_bucket(item.net_total())
}
