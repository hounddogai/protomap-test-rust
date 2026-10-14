use super::*;

use std::collections::BTreeMap;

use crate::egress::egress_override as egress_egress_override;

pub fn summarize_egress_summaries_for_entry(items: &[egress_egress_override::EgressSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_strict_segment_entries_from_summary(
    items: &[egress_egress_override::EgressSummary],
) -> Vec<StrictSegmentEntry> {
    let mut out: Vec<StrictSegmentEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_segment_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_strict_segment_entry(item: &StrictSegmentEntry) -> f64 {
    egress_egress_override::egress_summary_weight(2, true)
}
