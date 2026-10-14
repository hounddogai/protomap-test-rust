use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::summary as trust_domain_summary;

pub fn summarize_trust_domain_summaries_for_draft(
    items: &[trust_domain_summary::TrustDomainSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_credential_drafts_from_summary(
    items: &[trust_domain_summary::TrustDomainSummary],
) -> Vec<CredentialDraft> {
    let mut out: Vec<CredentialDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_draft(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_credential_draft(item: &CredentialDraft) -> String {
    trust_domain_summary::classify_trust_domain_summary(item.adjusted_exposure())
}
