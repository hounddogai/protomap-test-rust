use super::*;

use std::collections::BTreeMap;

use crate::flow_log::allocation as flow_log_allocation;

pub fn summarize_adaptive_flow_log_plans_for_reservation(
    items: &[flow_log_allocation::AdaptiveFlowLogPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_allowlist_reservations_from_plan(
    items: &[flow_log_allocation::AdaptiveFlowLogPlan],
) -> Vec<AllowlistReservation> {
    let mut out: Vec<AllowlistReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_reservation(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_allowlist_reservation(item: &AllowlistReservation) -> String {
    flow_log_allocation::classify_adaptive_flow_log_plan(item.net_exposure())
}
