use super::*;

use std::collections::BTreeMap;

use crate::health_check::check_segment as health_check_check_segment;

pub fn summarize_health_check_reviews_for_policy(
    items: &[health_check_check_segment::HealthCheckReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_tiered_dns_record_policies_from_review(
    items: &[health_check_check_segment::HealthCheckReview],
) -> Vec<TieredDnsRecordPolicy> {
    let mut out: Vec<TieredDnsRecordPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_dns_record_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_dns_record_policy(item: &TieredDnsRecordPolicy) -> String {
    health_check_check_segment::normalize_global_health_check_segment_code(&item.code)
}
