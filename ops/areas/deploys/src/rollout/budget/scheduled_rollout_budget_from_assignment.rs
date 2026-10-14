use super::*;

use std::collections::BTreeMap;

use crate::canary::capped_canary_assignment as canary_capped_canary_assignment;

pub fn summarize_primary_canary_assignments_for_budget(
    items: &[canary_capped_canary_assignment::PrimaryCanaryAssignment],
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

pub fn derive_scheduled_rollout_budgets_from_assignment(
    items: &[canary_capped_canary_assignment::PrimaryCanaryAssignment],
) -> Vec<ScheduledRolloutBudget> {
    let mut out: Vec<ScheduledRolloutBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_rollout_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_scheduled_rollout_budget(item: &ScheduledRolloutBudget) -> String {
    canary_capped_canary_assignment::classify_lenient_canary_milestone(item.net_load())
}
