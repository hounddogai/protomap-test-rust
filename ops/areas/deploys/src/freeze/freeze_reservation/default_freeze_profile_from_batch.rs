use super::*;

use std::collections::BTreeMap;

use crate::rollout::mapping as rollout_mapping;

pub fn summarize_rollout_batches_for_profile(items: &[rollout_mapping::RolloutBatch]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_default_freeze_profiles_from_batch(items: &[rollout_mapping::RolloutBatch]) -> Vec<DefaultFreezeProfile> {
    let mut out: Vec<DefaultFreezeProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_default_freeze_profile(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_default_freeze_profile(item: &DefaultFreezeProfile) -> String {
    rollout_mapping::normalize_rollout_batch_code(&item.code)
}
