use super::*;

use std::collections::BTreeMap;

use crate::postmortem::rule as postmortem_rule;

pub fn summarize_batched_postmortem_audits_for_plan(
    items: &[postmortem_rule::BatchedPostmortemAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_weighted_timeline_plans_from_audit(
    items: &[postmortem_rule::BatchedPostmortemAudit],
) -> Vec<WeightedTimelinePlan> {
    let mut out: Vec<WeightedTimelinePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_timeline_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_weighted_timeline_plan(item: &WeightedTimelinePlan) -> String {
    postmortem_rule::normalize_tiered_postmortem_cycle_code(&item.code)
}
