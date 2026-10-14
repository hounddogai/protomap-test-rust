use super::*;

use std::collections::BTreeMap;

use crate::token::baseline as token_baseline;

pub fn summarize_token_profiles_for_policy(items: &[token_baseline::TokenProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_identity_policies_from_profile(items: &[token_baseline::TokenProfile]) -> Vec<IdentityPolicy> {
    let mut out: Vec<IdentityPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_identity_policy(item: &IdentityPolicy) -> String {
    token_baseline::manual_token_proposal_bucket(item.weighted_exposure())
}
