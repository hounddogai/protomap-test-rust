use super::*;

use std::collections::BTreeMap;

use crate::flow_log::claim as flow_log_claim;

pub fn summarize_strict_flow_log_audits_for_audit(
    items: &[flow_log_claim::StrictFlowLogAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_adaptive_flow_log_audits_from_audit(
    items: &[flow_log_claim::StrictFlowLogAudit],
) -> Vec<AdaptiveFlowLogAudit> {
    let mut out: Vec<AdaptiveFlowLogAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_flow_log_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_adaptive_flow_log_audit(item: &AdaptiveFlowLogAudit) -> String {
    flow_log_claim::normalize_flow_log_claim_code(&item.code)
}
