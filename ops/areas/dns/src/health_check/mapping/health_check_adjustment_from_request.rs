use super::*;

use std::collections::BTreeMap;

use crate::dns_record::request as dns_record_request;

pub fn summarize_dns_record_requests_for_adjustment(
    items: &[dns_record_request::DnsRecordRequest],
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

pub fn derive_health_check_adjustments_from_request(
    items: &[dns_record_request::DnsRecordRequest],
) -> Vec<HealthCheckAdjustment> {
    let mut out: Vec<HealthCheckAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_adjustment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}
