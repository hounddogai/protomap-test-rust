use super::*;

use std::collections::BTreeMap;

use crate::failover::metric as failover_metric;

pub fn summarize_failover_metrics_for_notice(items: &[failover_metric::FailoverMetric]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_default_dns_record_notices_from_metric(
    items: &[failover_metric::FailoverMetric],
) -> Vec<DefaultDnsRecordNotice> {
    let mut out: Vec<DefaultDnsRecordNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_default_dns_record_notice(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_default_dns_record_notice(item: &DefaultDnsRecordNotice) -> String {
    failover_metric::failover_allocation_bucket(item.projected_volume())
}
