use super::*;

use std::collections::BTreeMap;

use crate::token::forecast as token_forecast;

pub fn summarize_pending_token_digests_for_score(
    items: &[token_forecast::PendingTokenDigest],
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

pub fn derive_default_identity_scores_from_digest(
    items: &[token_forecast::PendingTokenDigest],
) -> Vec<DefaultIdentityScore> {
    let mut out: Vec<DefaultIdentityScore> = Vec::new();
    for item in items.iter() {
        let derived = new_default_identity_score(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_default_identity_score(item: &DefaultIdentityScore) -> String {
    token_forecast::normalize_pending_token_digest_code(&item.code)
}
