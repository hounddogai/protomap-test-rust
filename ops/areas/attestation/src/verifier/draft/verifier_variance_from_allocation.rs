use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_allocation as nonce_nonce_allocation;

pub fn summarize_seasonal_nonce_allocations_for_variance(
    items: &[nonce_nonce_allocation::SeasonalNonceAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_verifier_variances_from_allocation(
    items: &[nonce_nonce_allocation::SeasonalNonceAllocation],
) -> Vec<VerifierVariance> {
    let mut out: Vec<VerifierVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_verifier_variance(item: &VerifierVariance) -> f64 {
    nonce_nonce_allocation::fallback_nonce_ledger_weight(4, true)
}
