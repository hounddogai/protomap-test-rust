use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_record as peering_peering_record;

pub fn summarize_global_peering_mappings_for_budget(
    items: &[peering_peering_record::GlobalPeeringMapping],
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

pub fn derive_rolling_flow_log_budgets_from_mapping(
    items: &[peering_peering_record::GlobalPeeringMapping],
) -> Vec<RollingFlowLogBudget> {
    let mut out: Vec<RollingFlowLogBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_flow_log_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_rolling_flow_log_budget(item: &RollingFlowLogBudget) -> String {
    peering_peering_record::classify_peering_preference(item.effective_exposure())
}
