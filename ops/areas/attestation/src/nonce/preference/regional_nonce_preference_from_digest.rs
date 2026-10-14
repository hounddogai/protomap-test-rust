use super::*;

use std::collections::BTreeMap;

use crate::quote::digest as quote_digest;

pub fn summarize_quote_digests_for_preference(items: &[quote_digest::QuoteDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_regional_nonce_preferences_from_digest(
    items: &[quote_digest::QuoteDigest],
) -> Vec<RegionalNoncePreference> {
    let mut out: Vec<RegionalNoncePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_nonce_preference(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_regional_nonce_preference(item: &RegionalNoncePreference) -> String {
    quote_digest::normalize_quote_digest_code(&item.code)
}
