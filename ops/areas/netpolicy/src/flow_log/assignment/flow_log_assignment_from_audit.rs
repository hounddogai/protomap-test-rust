use super::*;

use std::collections::BTreeMap;

use crate::flow_log::audit as flow_log_audit;

pub fn summarize_adaptive_flow_log_audits_for_assignment(
    items: &[flow_log_audit::AdaptiveFlowLogAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_flow_log_assignments_from_audit(
    items: &[flow_log_audit::AdaptiveFlowLogAudit],
) -> Vec<FlowLogAssignment> {
    let mut out: Vec<FlowLogAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_assignment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_flow_log_assignment_extra(item: &FlowLogAssignment) -> String {
    flow_log_audit::normalize_adaptive_flow_log_audit_code(&item.code)
}
