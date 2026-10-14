use super::*;

use std::collections::BTreeMap;

use crate::token::request as token_request;

pub fn summarize_global_token_requests_for_snapshot(
    items: &[token_request::GlobalTokenRequest],
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

pub fn derive_token_snapshots_from_request(items: &[token_request::GlobalTokenRequest]) -> Vec<TokenSnapshot> {
    let mut out: Vec<TokenSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_token_snapshot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_token_snapshot(item: &TokenSnapshot) -> f64 {
    token_request::capped_token_segment_weight(2, true)
}
