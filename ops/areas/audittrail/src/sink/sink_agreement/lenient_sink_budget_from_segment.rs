use super::*;

use std::collections::BTreeMap;

use crate::sink::segment as sink_segment;

pub fn summarize_batched_sink_segments_for_budget(items: &[sink_segment::BatchedSinkSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_lenient_sink_budgets_from_segment(items: &[sink_segment::BatchedSinkSegment]) -> Vec<LenientSinkBudget> {
    let mut out: Vec<LenientSinkBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_sink_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_lenient_sink_budget(item: &LenientSinkBudget) -> f64 {
    sink_segment::pending_sink_review_weight(4, true)
}
