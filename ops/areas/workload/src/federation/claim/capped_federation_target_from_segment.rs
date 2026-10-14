use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::segment as trust_domain_segment;

pub fn summarize_trust_domain_segments_for_target(
    items: &[trust_domain_segment::TrustDomainSegment],
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

pub fn derive_capped_federation_targets_from_segment(
    items: &[trust_domain_segment::TrustDomainSegment],
) -> Vec<CappedFederationTarget> {
    let mut out: Vec<CappedFederationTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_federation_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_capped_federation_target(item: &CappedFederationTarget) -> String {
    trust_domain_segment::normalize_trust_domain_schedule_code(&item.code)
}
