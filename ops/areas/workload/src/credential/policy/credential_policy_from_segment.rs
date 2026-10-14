use super::*;

use std::collections::BTreeMap;

use crate::token::token_schedule as token_token_schedule;

pub fn summarize_tiered_token_segments_for_policy(
    items: &[token_token_schedule::TieredTokenSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_credential_policies_from_segment(
    items: &[token_token_schedule::TieredTokenSegment],
) -> Vec<CredentialPolicy> {
    let mut out: Vec<CredentialPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_credential_policy(item: &CredentialPolicy) -> String {
    token_token_schedule::rolling_token_schedule_bucket(item.adjusted_value())
}
