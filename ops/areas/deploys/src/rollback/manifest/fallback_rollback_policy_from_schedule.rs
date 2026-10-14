use super::*;

use std::collections::BTreeMap;

use crate::rollback::exception as rollback_exception;

pub fn summarize_rollback_schedules_for_policy(
    items: &[rollback_exception::RollbackSchedule],
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

pub fn derive_fallback_rollback_policies_from_schedule(
    items: &[rollback_exception::RollbackSchedule],
) -> Vec<FallbackRollbackPolicy> {
    let mut out: Vec<FallbackRollbackPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_rollback_policy(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_fallback_rollback_policy(item: &FallbackRollbackPolicy) -> String {
    rollback_exception::classify_rollback_schedule(item.projected_value())
}
