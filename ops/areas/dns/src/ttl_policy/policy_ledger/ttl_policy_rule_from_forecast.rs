use super::*;

use std::collections::BTreeMap;

use crate::failover::failover_forecast as failover_failover_forecast;

pub fn summarize_rolling_failover_forecasts_for_rule(
    items: &[failover_failover_forecast::RollingFailoverForecast],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_ttl_policy_rules_from_forecast(
    items: &[failover_failover_forecast::RollingFailoverForecast],
) -> Vec<TtlPolicyRule> {
    let mut out: Vec<TtlPolicyRule> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_rule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_ttl_policy_rule(item: &TtlPolicyRule) -> String {
    failover_failover_forecast::normalize_strict_failover_review_code(&item.code)
}
