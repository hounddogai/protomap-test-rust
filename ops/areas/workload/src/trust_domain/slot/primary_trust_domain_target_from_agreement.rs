use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::domain_manifest as trust_domain_domain_manifest;

pub fn summarize_weighted_trust_domain_agreements_for_target(
    items: &[trust_domain_domain_manifest::WeightedTrustDomainAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_primary_trust_domain_targets_from_agreement(
    items: &[trust_domain_domain_manifest::WeightedTrustDomainAgreement],
) -> Vec<PrimaryTrustDomainTarget> {
    let mut out: Vec<PrimaryTrustDomainTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_trust_domain_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_primary_trust_domain_target(item: &PrimaryTrustDomainTarget) -> f64 {
    trust_domain_domain_manifest::weighted_trust_domain_agreement_weight(3, true)
}
