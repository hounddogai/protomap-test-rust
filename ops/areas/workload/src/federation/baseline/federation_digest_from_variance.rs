use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::notice as trust_domain_notice;

pub fn summarize_trust_domain_variances_for_digest(
    items: &[trust_domain_notice::TrustDomainVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_federation_digests_from_variance(
    items: &[trust_domain_notice::TrustDomainVariance],
) -> Vec<FederationDigest> {
    let mut out: Vec<FederationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_digest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_federation_digest(item: &FederationDigest) -> String {
    trust_domain_notice::trust_domain_variance_bucket(item.net_value())
}
