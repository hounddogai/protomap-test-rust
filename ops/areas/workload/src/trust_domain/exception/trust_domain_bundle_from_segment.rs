use super::*;

use std::collections::BTreeMap;

use crate::credential::plan as credential_plan;

pub fn summarize_credential_segments_for_bundle(items: &[credential_plan::CredentialSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_trust_domain_bundles_from_segment(
    items: &[credential_plan::CredentialSegment],
) -> Vec<TrustDomainBundle> {
    let mut out: Vec<TrustDomainBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_trust_domain_bundle(item: &TrustDomainBundle) -> String {
    credential_plan::credential_rule_bucket(item.projected_total())
}
