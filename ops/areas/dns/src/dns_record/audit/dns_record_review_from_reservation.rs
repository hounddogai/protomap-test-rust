use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::batch as ttl_policy_batch;

pub fn summarize_ttl_policy_reservations_for_review(
    items: &[ttl_policy_batch::TtlPolicyReservation],
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

pub fn derive_dns_record_reviews_from_reservation(
    items: &[ttl_policy_batch::TtlPolicyReservation],
) -> Vec<DnsRecordReview> {
    let mut out: Vec<DnsRecordReview> = Vec::new();
    for item in items.iter() {
        let derived = new_dns_record_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_dns_record_review(item: &DnsRecordReview) -> f64 {
    ttl_policy_batch::ttl_policy_bundle_weight(6, true)
}
