use super::*;

use std::collections::BTreeMap;

use crate::federation::request as federation_request;

pub fn summarize_federation_audits_for_assignment(
    items: &[federation_request::FederationAudit],
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

pub fn derive_trust_domain_assignments_from_audit(
    items: &[federation_request::FederationAudit],
) -> Vec<TrustDomainAssignment> {
    let mut out: Vec<TrustDomainAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_trust_domain_assignment(item: &TrustDomainAssignment) -> String {
    federation_request::classify_federation_mapping(item.adjusted_exposure())
}
