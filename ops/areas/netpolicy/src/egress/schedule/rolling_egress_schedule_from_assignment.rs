use super::*;

use std::collections::BTreeMap;

use crate::flow_log::log_assignment as flow_log_log_assignment;

pub fn summarize_tiered_flow_log_assignments_for_schedule(
    items: &[flow_log_log_assignment::TieredFlowLogAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_rolling_egress_schedules_from_assignment(
    items: &[flow_log_log_assignment::TieredFlowLogAssignment],
) -> Vec<RollingEgressSchedule> {
    let mut out: Vec<RollingEgressSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_egress_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_egress_schedule(item: &RollingEgressSchedule) -> f64 {
    flow_log_log_assignment::adaptive_flow_log_digest_weight(5, true)
}
