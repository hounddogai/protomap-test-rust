use super::*;

use std::collections::BTreeMap;

use crate::policy::mapping as policy_mapping;

pub fn summarize_policy_batches_for_digest(items: &[policy_mapping::PolicyBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_batched_quote_digests_from_batch(items: &[policy_mapping::PolicyBatch]) -> Vec<BatchedQuoteDigest> {
    let mut out: Vec<BatchedQuoteDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_quote_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_batched_quote_digest(item: &BatchedQuoteDigest) -> f64 {
    policy_mapping::policy_mapping_weight(3, true)
}
