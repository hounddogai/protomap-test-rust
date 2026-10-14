use super::*;

use std::collections::BTreeMap;

use crate::redaction::batch as redaction_batch;

pub fn summarize_redaction_cycles_for_mapping(items: &[redaction_batch::RedactionCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_rolling_sink_mappings_from_cycle(items: &[redaction_batch::RedactionCycle]) -> Vec<RollingSinkMapping> {
    let mut out: Vec<RollingSinkMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_sink_mapping(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_sink_mapping(item: &RollingSinkMapping) -> f64 {
    redaction_batch::primary_redaction_allocation_weight(9, true)
}
