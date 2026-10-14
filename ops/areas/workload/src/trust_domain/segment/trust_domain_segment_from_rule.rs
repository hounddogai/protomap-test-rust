use super::*;

use std::collections::BTreeMap;

use crate::credential::preference as credential_preference;

pub fn summarize_tiered_credential_rules_for_segment(
    items: &[credential_preference::TieredCredentialRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_trust_domain_segments_from_rule(
    items: &[credential_preference::TieredCredentialRule],
) -> Vec<TrustDomainSegment> {
    let mut out: Vec<TrustDomainSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_trust_domain_segment(item: &TrustDomainSegment) -> String {
    credential_preference::classify_weighted_credential_preference(item.measured_value())
}
