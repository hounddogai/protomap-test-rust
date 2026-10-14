use super::*;

use std::collections::BTreeMap;

use crate::redaction::batch as redaction_batch;

pub fn summarize_redaction_cycles_for_adjustment(items: &[redaction_batch::RedactionCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_pending_redaction_adjustments_from_cycle(
    items: &[redaction_batch::RedactionCycle],
) -> Vec<PendingRedactionAdjustment> {
    let mut out: Vec<PendingRedactionAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_redaction_adjustment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_pending_redaction_adjustment(item: &PendingRedactionAdjustment) -> f64 {
    redaction_batch::primary_redaction_allocation_weight(7, true)
}
