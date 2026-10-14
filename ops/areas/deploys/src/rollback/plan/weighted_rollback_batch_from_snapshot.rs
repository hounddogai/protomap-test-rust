use super::*;

use std::collections::BTreeMap;

use crate::rollout::digest as rollout_digest;

pub fn summarize_adaptive_rollout_snapshots_for_batch(
    items: &[rollout_digest::AdaptiveRolloutSnapshot],
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

pub fn derive_weighted_rollback_batches_from_snapshot(
    items: &[rollout_digest::AdaptiveRolloutSnapshot],
) -> Vec<WeightedRollbackBatch> {
    let mut out: Vec<WeightedRollbackBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rollback_batch(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_weighted_rollback_batch(item: &WeightedRollbackBatch) -> String {
    rollout_digest::normalize_rolling_rollout_digest_code(&item.code)
}
