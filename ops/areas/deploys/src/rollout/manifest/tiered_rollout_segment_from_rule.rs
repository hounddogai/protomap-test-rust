use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_assignment as freeze_freeze_assignment;

pub fn summarize_adaptive_freeze_rules_for_segment(
    items: &[freeze_freeze_assignment::AdaptiveFreezeRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_rollout_segments_from_rule(
    items: &[freeze_freeze_assignment::AdaptiveFreezeRule],
) -> Vec<TieredRolloutSegment> {
    let mut out: Vec<TieredRolloutSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_rollout_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_rollout_segment(item: &TieredRolloutSegment) -> String {
    freeze_freeze_assignment::scheduled_freeze_assignment_bucket(item.measured_value())
}
