use super::*;

use std::collections::BTreeMap;

use crate::forecast::forecast_group as forecast_forecast_group;

pub fn summarize_default_forecast_drafts_for_quota(
    items: &[forecast_forecast_group::DefaultForecastDraft],
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

pub fn derive_primary_limit_quotas_from_draft(
    items: &[forecast_forecast_group::DefaultForecastDraft],
) -> Vec<PrimaryLimitQuota> {
    let mut out: Vec<PrimaryLimitQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_limit_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_primary_limit_quota(item: &PrimaryLimitQuota) -> String {
    forecast_forecast_group::classify_default_forecast_notice(item.net_volume())
}
