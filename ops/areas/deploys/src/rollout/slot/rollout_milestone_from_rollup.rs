use super::*;

use std::collections::BTreeMap;

use crate::freeze::rollup as freeze_rollup;

pub fn summarize_freeze_rollups_for_milestone(items: &[freeze_rollup::FreezeRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rollout_milestones_from_rollup(items: &[freeze_rollup::FreezeRollup]) -> Vec<RolloutMilestone> {
    let mut out: Vec<RolloutMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_rollout_milestone(item: &RolloutMilestone) -> String {
    freeze_rollup::classify_freeze_rollup(item.weighted_exposure())
}
