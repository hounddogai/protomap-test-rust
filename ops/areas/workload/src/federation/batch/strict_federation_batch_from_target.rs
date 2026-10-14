use super::*;

use std::collections::BTreeMap;

use crate::federation::targets as federation_targets;

pub fn summarize_federation_targets_for_batch(items: &[federation_targets::FederationTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_strict_federation_batches_from_target(
    items: &[federation_targets::FederationTarget],
) -> Vec<StrictFederationBatch> {
    let mut out: Vec<StrictFederationBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_federation_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_strict_federation_batch(item: &StrictFederationBatch) -> String {
    federation_targets::federation_target_bucket(item.effective_load())
}
