use super::*;

use std::collections::BTreeMap;

use crate::quote::claim as quote_claim;

pub fn summarize_quote_claims_for_variance(items: &[quote_claim::QuoteClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_lenient_nonce_variances_from_claim(items: &[quote_claim::QuoteClaim]) -> Vec<LenientNonceVariance> {
    let mut out: Vec<LenientNonceVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_nonce_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_nonce_variance(item: &LenientNonceVariance) -> f64 {
    quote_claim::fallback_quote_threshold_weight(8, true)
}
