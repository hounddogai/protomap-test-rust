use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::threshold as trust_domain_threshold;

pub fn summarize_trust_domain_groups_for_bundle(
    items: &[trust_domain_threshold::TrustDomainGroup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_federation_bundles_from_group(
    items: &[trust_domain_threshold::TrustDomainGroup],
) -> Vec<FederationBundle> {
    let mut out: Vec<FederationBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_federation_bundle(item: &FederationBundle) -> String {
    trust_domain_threshold::trust_domain_group_bucket(item.measured_value())
}
