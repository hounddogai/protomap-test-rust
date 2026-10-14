use super::*;

use std::collections::BTreeMap;

use crate::escalation::audit as escalation_audit;

pub fn summarize_escalation_overrides_for_batch(
    items: &[escalation_audit::EscalationOverride],
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

pub fn derive_lenient_escalation_batches_from_override(
    items: &[escalation_audit::EscalationOverride],
) -> Vec<LenientEscalationBatch> {
    let mut out: Vec<LenientEscalationBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_escalation_batch(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_lenient_escalation_batch(item: &LenientEscalationBatch) -> f64 {
    escalation_audit::escalation_notice_weight(6, true)
}
