use super::*;

use std::collections::BTreeMap;

use crate::flow_log::log_assignment as flow_log_log_assignment;

pub fn summarize_seasonal_flow_log_assignments_for_plan(
    items: &[flow_log_log_assignment::SeasonalFlowLogAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_flow_log_plans_from_assignment(
    items: &[flow_log_log_assignment::SeasonalFlowLogAssignment],
) -> Vec<FlowLogPlan> {
    let mut out: Vec<FlowLogPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_plan(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_flow_log_plan(item: &FlowLogPlan) -> f64 {
    flow_log_log_assignment::manual_flow_log_policy_weight(5, true)
}
