use super::*;

use std::collections::BTreeMap;

use crate::quote::group as quote_group;

pub fn summarize_adaptive_quote_groups_for_target(items: &[quote_group::AdaptiveQuoteGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_pending_nonce_targets_from_group(items: &[quote_group::AdaptiveQuoteGroup]) -> Vec<PendingNonceTarget> {
    let mut out: Vec<PendingNonceTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_nonce_target(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_pending_nonce_target(item: &PendingNonceTarget) -> String {
    quote_group::quote_plan_bucket(item.weighted_value())
}
