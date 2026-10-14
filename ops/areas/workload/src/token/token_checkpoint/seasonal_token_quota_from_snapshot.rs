use super::*;

use std::collections::BTreeMap;

use crate::token::snapshot as token_snapshot;

pub fn summarize_token_snapshots_for_quota(items: &[token_snapshot::TokenSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_seasonal_token_quotas_from_snapshot(items: &[token_snapshot::TokenSnapshot]) -> Vec<SeasonalTokenQuota> {
    let mut out: Vec<SeasonalTokenQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_token_quota(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_seasonal_token_quota(item: &SeasonalTokenQuota) -> String {
    token_snapshot::token_forecast_bucket(item.adjusted_exposure())
}
