use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::notice as ttl_policy_notice;

pub fn summarize_ttl_policy_notices_for_milestone(
    items: &[ttl_policy_notice::TtlPolicyNotice],
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

pub fn derive_dns_record_milestones_from_notice(
    items: &[ttl_policy_notice::TtlPolicyNotice],
) -> Vec<DnsRecordMilestone> {
    let mut out: Vec<DnsRecordMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_milestone(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_dns_record_milestone(item: &DnsRecordMilestone) -> f64 {
    ttl_policy_notice::ttl_policy_notice_weight(4, true)
}
