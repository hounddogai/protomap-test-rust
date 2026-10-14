use super::*;

use std::collections::BTreeMap;

use crate::flow_log::snapshot as flow_log_snapshot;

pub fn summarize_flow_log_snapshots_for_budget(items: &[flow_log_snapshot::FlowLogSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_capped_peering_budgets_from_snapshot(
    items: &[flow_log_snapshot::FlowLogSnapshot],
) -> Vec<CappedPeeringBudget> {
    let mut out: Vec<CappedPeeringBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_peering_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_capped_peering_budget(item: &CappedPeeringBudget) -> f64 {
    flow_log_snapshot::flow_log_reservation_weight(8, true)
}
