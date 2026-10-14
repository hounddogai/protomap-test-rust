use super::*;

use std::collections::BTreeMap;

use crate::nonce::summary as nonce_summary;

pub fn summarize_nonce_summaries_for_item(items: &[nonce_summary::NonceSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_primary_policy_items_from_summary(items: &[nonce_summary::NonceSummary]) -> Vec<PrimaryPolicyItem> {
    let mut out: Vec<PrimaryPolicyItem> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_policy_item(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_policy_item(item: &PrimaryPolicyItem) -> String {
    nonce_summary::pending_nonce_estimate_bucket(item.adjusted_value())
}
