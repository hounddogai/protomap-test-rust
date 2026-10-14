use super::*;

use std::collections::BTreeMap;

use crate::token::profile as token_profile;

pub fn summarize_weighted_token_exceptions_for_proposal(
    items: &[token_profile::WeightedTokenException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_weighted_binding_proposals_from_exception(
    items: &[token_profile::WeightedTokenException],
) -> Vec<WeightedBindingProposal> {
    let mut out: Vec<WeightedBindingProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_binding_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_weighted_binding_proposal(item: &WeightedBindingProposal) -> String {
    token_profile::classify_rolling_token_profile(item.projected_load())
}
