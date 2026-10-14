use super::*;

use std::collections::BTreeMap;

use crate::dns_record::preference as dns_record_preference;

pub fn summarize_dns_record_batches_for_milestone(
    items: &[dns_record_preference::DnsRecordBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_health_check_milestones_from_batch(
    items: &[dns_record_preference::DnsRecordBatch],
) -> Vec<HealthCheckMilestone> {
    let mut out: Vec<HealthCheckMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_milestone(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_health_check_milestone(item: &HealthCheckMilestone) -> String {
    dns_record_preference::normalize_dns_record_entry_code(&item.code)
}
