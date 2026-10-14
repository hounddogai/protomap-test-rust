use super::*;

use std::collections::BTreeMap;

use crate::federation::draft as federation_draft;

pub fn summarize_rolling_federation_requests_for_draft(
    items: &[federation_draft::RollingFederationRequest],
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

pub fn derive_trust_domain_drafts_from_request(
    items: &[federation_draft::RollingFederationRequest],
) -> Vec<TrustDomainDraft> {
    let mut out: Vec<TrustDomainDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_trust_domain_draft(item: &TrustDomainDraft) -> String {
    federation_draft::normalize_rolling_federation_request_code(&item.code)
}
