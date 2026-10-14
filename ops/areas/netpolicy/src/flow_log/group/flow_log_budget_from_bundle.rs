use super::*;

use std::collections::BTreeMap;

use crate::flow_log::bundle as flow_log_bundle;

pub fn summarize_flow_log_bundles_for_budget(items: &[flow_log_bundle::FlowLogBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_flow_log_budgets_from_bundle(items: &[flow_log_bundle::FlowLogBundle]) -> Vec<FlowLogBudget> {
    let mut out: Vec<FlowLogBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_flow_log_budget(item: &FlowLogBudget) -> String {
    flow_log_bundle::classify_flow_log_bundle(item.weighted_exposure())
}
