use super::*;

use std::collections::BTreeMap;

use crate::nonce::tier as nonce_tier;

pub fn summarize_nonce_baselines_for_policy(items: &[nonce_tier::NonceBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_quote_policies_from_baseline(items: &[nonce_tier::NonceBaseline]) -> Vec<QuotePolicy> {
    let mut out: Vec<QuotePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_quote_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_quote_policy_extra(item: &QuotePolicy) -> String {
    nonce_tier::classify_nonce_baseline(item.adjusted_exposure())
}
