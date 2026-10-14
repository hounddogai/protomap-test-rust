use super::*;

use std::collections::BTreeMap;

use crate::nonce::policy as nonce_policy;

pub fn summarize_lenient_nonce_metrics_for_schedule(
    items: &[nonce_policy::LenientNonceMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_policy_schedules_from_metric(items: &[nonce_policy::LenientNonceMetric]) -> Vec<PolicySchedule> {
    let mut out: Vec<PolicySchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_schedule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_policy_schedule(item: &PolicySchedule) -> String {
    nonce_policy::scheduled_nonce_rule_bucket(item.weighted_load())
}
