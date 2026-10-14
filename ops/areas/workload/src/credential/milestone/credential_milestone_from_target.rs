use super::*;

use std::collections::BTreeMap;

use crate::token::quota as token_quota;

pub fn summarize_token_targets_for_milestone(items: &[token_quota::TokenTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_credential_milestones_from_target(items: &[token_quota::TokenTarget]) -> Vec<CredentialMilestone> {
    let mut out: Vec<CredentialMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_milestone(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_credential_milestone(item: &CredentialMilestone) -> f64 {
    token_quota::token_override_weight(1, true)
}
