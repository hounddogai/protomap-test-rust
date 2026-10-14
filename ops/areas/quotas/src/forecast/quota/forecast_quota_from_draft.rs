use super::*;

use std::collections::BTreeMap;

use crate::limit::request as limit_request;

pub fn summarize_limit_drafts_for_quota(items: &[limit_request::LimitDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_forecast_quotas_from_draft(items: &[limit_request::LimitDraft]) -> Vec<ForecastQuota> {
    let mut out: Vec<ForecastQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_forecast_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_forecast_quota(item: &ForecastQuota) -> f64 {
    limit_request::limit_audit_weight(5, true)
}
