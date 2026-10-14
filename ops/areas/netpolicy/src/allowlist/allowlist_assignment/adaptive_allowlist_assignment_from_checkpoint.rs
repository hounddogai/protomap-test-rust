use super::*;

use std::collections::BTreeMap;

use crate::flow_log::digest as flow_log_digest;

pub fn summarize_rolling_flow_log_checkpoints_for_assignment(
    items: &[flow_log_digest::RollingFlowLogCheckpoint],
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

pub fn derive_adaptive_allowlist_assignments_from_checkpoint(
    items: &[flow_log_digest::RollingFlowLogCheckpoint],
) -> Vec<AdaptiveAllowlistAssignment> {
    let mut out: Vec<AdaptiveAllowlistAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_allowlist_assignment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_adaptive_allowlist_assignment(item: &AdaptiveAllowlistAssignment) -> String {
    flow_log_digest::legacy_flow_log_proposal_bucket(item.projected_load())
}
