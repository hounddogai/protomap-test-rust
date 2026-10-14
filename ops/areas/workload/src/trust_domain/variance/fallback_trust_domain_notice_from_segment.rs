use super::*;

use std::collections::BTreeMap;

use crate::federation::schedule as federation_schedule;

pub fn summarize_federation_segments_for_notice(
    items: &[federation_schedule::FederationSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_fallback_trust_domain_notices_from_segment(
    items: &[federation_schedule::FederationSegment],
) -> Vec<FallbackTrustDomainNotice> {
    let mut out: Vec<FallbackTrustDomainNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_trust_domain_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_fallback_trust_domain_notice(item: &FallbackTrustDomainNotice) -> String {
    federation_schedule::strict_federation_profile_bucket(item.adjusted_value())
}
