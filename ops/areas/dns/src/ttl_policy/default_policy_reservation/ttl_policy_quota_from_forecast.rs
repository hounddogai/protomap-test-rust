use super::*;

use std::collections::BTreeMap;

use crate::delegation::rollup as delegation_rollup;

pub fn summarize_delegation_forecasts_for_quota(
    items: &[delegation_rollup::DelegationForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_ttl_policy_quotas_from_forecast(items: &[delegation_rollup::DelegationForecast]) -> Vec<TtlPolicyQuota> {
    let mut out: Vec<TtlPolicyQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_quota(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_ttl_policy_quota(item: &TtlPolicyQuota) -> String {
    delegation_rollup::classify_delegation_forecast(item.net_volume())
}
