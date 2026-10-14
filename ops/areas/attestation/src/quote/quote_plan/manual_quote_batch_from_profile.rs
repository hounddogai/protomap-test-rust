use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_profile as nonce_nonce_profile;

pub fn summarize_adaptive_nonce_profiles_for_batch(
    items: &[nonce_nonce_profile::AdaptiveNonceProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_manual_quote_batches_from_profile(
    items: &[nonce_nonce_profile::AdaptiveNonceProfile],
) -> Vec<ManualQuoteBatch> {
    let mut out: Vec<ManualQuoteBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_quote_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_manual_quote_batch(item: &ManualQuoteBatch) -> f64 {
    nonce_nonce_profile::adaptive_nonce_profile_weight(8, true)
}
