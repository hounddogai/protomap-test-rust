use super::*;

use std::collections::BTreeMap;

use crate::health_check::notice as health_check_notice;

pub fn summarize_health_check_templates_for_variance(
    items: &[health_check_notice::HealthCheckTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_dns_record_variances_from_template(
    items: &[health_check_notice::HealthCheckTemplate],
) -> Vec<DnsRecordVariance> {
    let mut out: Vec<DnsRecordVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_dns_record_variance_extra(item: &DnsRecordVariance) -> String {
    health_check_notice::normalize_health_check_notice_code(&item.code)
}
