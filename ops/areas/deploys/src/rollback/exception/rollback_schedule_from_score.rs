use super::*;

use std::collections::BTreeMap;

use crate::approval::score as approval_score;

pub fn summarize_approval_scores_for_schedule(items: &[approval_score::ApprovalScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_rollback_schedules_from_score(items: &[approval_score::ApprovalScore]) -> Vec<RollbackSchedule> {
    let mut out: Vec<RollbackSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_schedule(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_rollback_schedule(item: &RollbackSchedule) -> String {
    approval_score::approval_score_bucket(item.measured_value())
}
