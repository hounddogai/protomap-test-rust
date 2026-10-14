use super::*;

use std::collections::BTreeMap;

use crate::token::digest as token_digest;

pub fn summarize_token_groups_for_draft(items: &[token_digest::TokenGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_token_drafts_from_group(items: &[token_digest::TokenGroup]) -> Vec<TokenDraft> {
    let mut out: Vec<TokenDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_token_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_token_draft(item: &TokenDraft) -> String {
    token_digest::rolling_token_digest_bucket(item.adjusted_load())
}
