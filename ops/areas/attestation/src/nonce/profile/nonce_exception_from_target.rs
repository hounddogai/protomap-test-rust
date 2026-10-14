use super::*;

use std::collections::BTreeMap;

use crate::nonce::targets as nonce_targets;

pub fn summarize_tiered_nonce_targets_for_exception(
    items: &[nonce_targets::TieredNonceTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_nonce_exceptions_from_target(items: &[nonce_targets::TieredNonceTarget]) -> Vec<NonceException> {
    let mut out: Vec<NonceException> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_nonce_exception(item: &NonceException) -> String {
    nonce_targets::tiered_nonce_target_bucket(item.effective_total())
}
