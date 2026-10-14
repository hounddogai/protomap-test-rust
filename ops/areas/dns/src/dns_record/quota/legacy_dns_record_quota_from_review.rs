use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_review as zone_zone_review;

pub fn summarize_default_zone_reviews_for_quota(
    items: &[zone_zone_review::DefaultZoneReview],
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

pub fn derive_legacy_dns_record_quotas_from_review(
    items: &[zone_zone_review::DefaultZoneReview],
) -> Vec<LegacyDnsRecordQuota> {
    let mut out: Vec<LegacyDnsRecordQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_dns_record_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_legacy_dns_record_quota(item: &LegacyDnsRecordQuota) -> String {
    zone_zone_review::classify_zone_entry(item.measured_value())
}
