use super::*;

use std::collections::BTreeMap;

use crate::canary::checkpoint as canary_checkpoint;

pub fn summarize_canary_summaries_for_audit(items: &[canary_checkpoint::CanarySummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_approval_audits_from_summary(items: &[canary_checkpoint::CanarySummary]) -> Vec<ApprovalAudit> {
    let mut out: Vec<ApprovalAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_audit(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_approval_audit(item: &ApprovalAudit) -> f64 {
    canary_checkpoint::canary_forecast_weight(3, true)
}
