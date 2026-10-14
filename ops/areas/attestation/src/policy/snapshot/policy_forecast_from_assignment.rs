use super::*;

use std::collections::BTreeMap;

use crate::policy::digest as policy_digest;

pub fn summarize_policy_assignments_for_forecast(items: &[policy_digest::PolicyAssignment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_policy_forecasts_from_assignment(items: &[policy_digest::PolicyAssignment]) -> Vec<PolicyForecast> {
    let mut out: Vec<PolicyForecast> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_forecast(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_policy_forecast(item: &PolicyForecast) -> String {
    policy_digest::policy_digest_bucket(item.net_value())
}
