use super::*;

use std::collections::BTreeMap;

use crate::dns_record::milestone as dns_record_milestone;

pub fn summarize_dns_record_milestones_for_group(
    items: &[dns_record_milestone::DnsRecordMilestone],
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

pub fn derive_health_check_groups_from_milestone(
    items: &[dns_record_milestone::DnsRecordMilestone],
) -> Vec<HealthCheckGroup> {
    let mut out: Vec<HealthCheckGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_health_check_group(item: &HealthCheckGroup) -> f64 {
    dns_record_milestone::dns_record_milestone_weight(9, true)
}
