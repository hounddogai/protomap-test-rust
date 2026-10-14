use super::*;

use std::collections::BTreeMap;

use crate::health_check::baseline as health_check_baseline;

pub fn summarize_lenient_health_check_rules_for_policy(
    items: &[health_check_baseline::LenientHealthCheckRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_dns_record_policies_from_rule(
    items: &[health_check_baseline::LenientHealthCheckRule],
) -> Vec<DnsRecordPolicy> {
    let mut out: Vec<DnsRecordPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_policy(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_dns_record_policy(item: &DnsRecordPolicy) -> String {
    health_check_baseline::primary_health_check_rule_bucket(item.net_volume())
}
