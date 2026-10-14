use super::*;

use std::collections::BTreeMap;

use crate::dns_record::milestone as dns_record_milestone;

pub fn summarize_dns_record_milestones_for_reservation(
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

pub fn derive_manual_ttl_policy_reservations_from_milestone(
    items: &[dns_record_milestone::DnsRecordMilestone],
) -> Vec<ManualTtlPolicyReservation> {
    let mut out: Vec<ManualTtlPolicyReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_ttl_policy_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_manual_ttl_policy_reservation(item: &ManualTtlPolicyReservation) -> f64 {
    dns_record_milestone::rolling_dns_record_item_weight(9, true)
}
