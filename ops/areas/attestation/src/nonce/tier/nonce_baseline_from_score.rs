use super::*;

use std::collections::BTreeMap;

use crate::nonce::variance as nonce_variance;

pub fn summarize_nonce_scores_for_baseline(items: &[nonce_variance::NonceScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_nonce_baselines_from_score(items: &[nonce_variance::NonceScore]) -> Vec<NonceBaseline> {
    let mut out: Vec<NonceBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_nonce_baseline(item: &NonceBaseline) -> f64 {
    nonce_variance::nonce_variance_weight(2, true)
}
