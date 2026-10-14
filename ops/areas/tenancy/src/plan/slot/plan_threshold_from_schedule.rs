use super::*;

use std::collections::BTreeMap;

use crate::suspension::quota as suspension_quota;

pub fn summarize_pending_suspension_schedules_for_threshold(
    items: &[suspension_quota::PendingSuspensionSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_plan_thresholds_from_schedule(
    items: &[suspension_quota::PendingSuspensionSchedule],
) -> Vec<PlanThreshold> {
    let mut out: Vec<PlanThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_threshold(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_plan_threshold(item: &PlanThreshold) -> String {
    suspension_quota::classify_pending_suspension_schedule(item.effective_value())
}
