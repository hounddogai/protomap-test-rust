use super::*;

use std::collections::BTreeMap;

use crate::suspension::group as suspension_group;

pub fn summarize_global_suspension_batches_for_tier(
    items: &[suspension_group::GlobalSuspensionBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_regional_isolation_tiers_from_batch(
    items: &[suspension_group::GlobalSuspensionBatch],
) -> Vec<RegionalIsolationTier> {
    let mut out: Vec<RegionalIsolationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_isolation_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_regional_isolation_tier(item: &RegionalIsolationTier) -> f64 {
    suspension_group::global_suspension_batch_weight(7, true)
}
