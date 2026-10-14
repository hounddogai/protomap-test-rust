use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_window as freeze_freeze_window;

pub fn summarize_regional_freeze_snapshots_for_score(
    items: &[freeze_freeze_window::RegionalFreezeSnapshot],
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

pub fn derive_scheduled_rollback_scores_from_snapshot(
    items: &[freeze_freeze_window::RegionalFreezeSnapshot],
) -> Vec<ScheduledRollbackScore> {
    let mut out: Vec<ScheduledRollbackScore> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_rollback_score(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_rollback_score(item: &ScheduledRollbackScore) -> String {
    freeze_freeze_window::classify_freeze_adjustment(item.projected_value())
}
