use super::*;

use std::collections::BTreeMap;

use crate::flow_log::assignment as flow_log_assignment;

pub fn summarize_global_flow_log_checkpoints_for_metric(
    items: &[flow_log_assignment::GlobalFlowLogCheckpoint],
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

pub fn derive_peering_metrics_from_checkpoint(
    items: &[flow_log_assignment::GlobalFlowLogCheckpoint],
) -> Vec<PeeringMetric> {
    let mut out: Vec<PeeringMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_peering_metric(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_peering_metric(item: &PeeringMetric) -> String {
    flow_log_assignment::normalize_tiered_flow_log_audit_code(&item.code)
}
