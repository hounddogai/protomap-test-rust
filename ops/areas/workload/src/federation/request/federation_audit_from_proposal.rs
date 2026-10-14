use super::*;

use std::collections::BTreeMap;

use crate::token::quota as token_quota;

pub fn summarize_token_proposals_for_audit(items: &[token_quota::TokenProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_federation_audits_from_proposal(items: &[token_quota::TokenProposal]) -> Vec<FederationAudit> {
    let mut out: Vec<FederationAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_federation_audit(item: &FederationAudit) -> String {
    token_quota::normalize_token_quota_code(&item.code)
}
