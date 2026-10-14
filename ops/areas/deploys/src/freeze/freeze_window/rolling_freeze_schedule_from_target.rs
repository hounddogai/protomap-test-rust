use super::*;

use std::collections::BTreeMap;

use crate::approval::adaptive_approval_quota as approval_adaptive_approval_quota;

pub fn summarize_approval_targets_for_schedule(
    items: &[approval_adaptive_approval_quota::ApprovalTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_rolling_freeze_schedules_from_target(
    items: &[approval_adaptive_approval_quota::ApprovalTarget],
) -> Vec<RollingFreezeSchedule> {
    let mut out: Vec<RollingFreezeSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_freeze_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_rolling_freeze_schedule_extra(item: &RollingFreezeSchedule) -> String {
    approval_adaptive_approval_quota::tiered_approval_quota_bucket(item.adjusted_total())
}
