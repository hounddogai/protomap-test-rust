use super::*;

use std::collections::BTreeMap;

use crate::dns_record::request as dns_record_request;

pub fn summarize_dns_record_requests_for_template(
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

pub fn derive_zone_templates_from_request(items: &[dns_record_request::DnsRecordRequest]) -> Vec<ZoneTemplate> {
    let mut out: Vec<ZoneTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_template(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}
