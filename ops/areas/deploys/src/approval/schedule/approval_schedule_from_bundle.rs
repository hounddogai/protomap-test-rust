use super::*;

use std::collections::BTreeMap;

use crate::rollout::budget as rollout_budget;

pub fn summarize_default_rollout_bundles_for_schedule(
    items: &[rollout_budget::DefaultRolloutBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_approval_schedules_from_bundle(items: &[rollout_budget::DefaultRolloutBundle]) -> Vec<ApprovalSchedule> {
    let mut out: Vec<ApprovalSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_schedule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_approval_schedule(item: &ApprovalSchedule) -> String {
    rollout_budget::normalize_default_rollout_bundle_code(&item.code)
}
