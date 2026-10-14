use super::*;

use std::collections::BTreeMap;

use crate::sink::allocation as sink_allocation;

pub fn summarize_sink_allocations_for_variance(items: &[sink_allocation::SinkAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_redaction_variances_from_allocation(items: &[sink_allocation::SinkAllocation]) -> Vec<RedactionVariance> {
    let mut out: Vec<RedactionVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_variance(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_redaction_variance(item: &RedactionVariance) -> String {
    sink_allocation::tiered_sink_ledger_bucket(item.net_total())
}
