use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::metric as ttl_policy_metric;

pub fn summarize_ttl_policy_metrics_for_notice(items: &[ttl_policy_metric::TtlPolicyMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_health_check_notices_from_metric(items: &[ttl_policy_metric::TtlPolicyMetric]) -> Vec<HealthCheckNotice> {
    let mut out: Vec<HealthCheckNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_health_check_notice(item: &HealthCheckNotice) -> String {
    ttl_policy_metric::ttl_policy_profile_bucket(item.weighted_total())
}
