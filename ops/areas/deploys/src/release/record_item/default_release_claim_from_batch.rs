use super::*;

use std::collections::BTreeMap;

use crate::canary::batch as canary_batch;

pub fn summarize_canary_batches_for_claim(items: &[canary_batch::CanaryBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_default_release_claims_from_batch(items: &[canary_batch::CanaryBatch]) -> Vec<DefaultReleaseClaim> {
    let mut out: Vec<DefaultReleaseClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_default_release_claim(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_default_release_claim(item: &DefaultReleaseClaim) -> f64 {
    canary_batch::canary_snapshot_weight(8, true)
}
