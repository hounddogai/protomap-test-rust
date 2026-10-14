use super::*;

use std::collections::BTreeMap;

use crate::release::snapshot as release_snapshot;

pub fn summarize_release_thresholds_for_milestone(
    items: &[release_snapshot::ReleaseThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_adaptive_rollback_milestones_from_threshold(
    items: &[release_snapshot::ReleaseThreshold],
) -> Vec<AdaptiveRollbackMilestone> {
    let mut out: Vec<AdaptiveRollbackMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_rollback_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_adaptive_rollback_milestone(item: &AdaptiveRollbackMilestone) -> String {
    release_snapshot::classify_release_snapshot(item.weighted_total())
}
