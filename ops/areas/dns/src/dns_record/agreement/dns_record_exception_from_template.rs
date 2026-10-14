use super::*;

use std::collections::BTreeMap;

use crate::dns_record::variance as dns_record_variance;

pub fn summarize_manual_dns_record_templates_for_exception(
    items: &[dns_record_variance::ManualDnsRecordTemplate],
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

pub fn derive_dns_record_exceptions_from_template(
    items: &[dns_record_variance::ManualDnsRecordTemplate],
) -> Vec<DnsRecordException> {
    let mut out: Vec<DnsRecordException> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_dns_record_exception_extra(item: &DnsRecordException) -> String {
    dns_record_variance::manual_dns_record_template_bucket(item.adjusted_value())
}
