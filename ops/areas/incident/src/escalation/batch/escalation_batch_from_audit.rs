use super::*;

use std::collections::BTreeMap;

use crate::postmortem::rollup as postmortem_rollup;

pub fn summarize_postmortem_audits_for_batch(items: &[postmortem_rollup::PostmortemAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_escalation_batches_from_audit(items: &[postmortem_rollup::PostmortemAudit]) -> Vec<EscalationBatch> {
    let mut out: Vec<EscalationBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_batch(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_escalation_batch(item: &EscalationBatch) -> f64 {
    postmortem_rollup::postmortem_exception_weight(7, true)
}
