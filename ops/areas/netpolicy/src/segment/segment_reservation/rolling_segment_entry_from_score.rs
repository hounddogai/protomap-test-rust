use super::*;

use std::collections::BTreeMap;

use crate::egress::entry as egress_entry;

pub fn summarize_primary_egress_scores_for_entry(items: &[egress_entry::PrimaryEgressScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rolling_segment_entries_from_score(
    items: &[egress_entry::PrimaryEgressScore],
) -> Vec<RollingSegmentEntry> {
    let mut out: Vec<RollingSegmentEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_segment_entry(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_rolling_segment_entry(item: &RollingSegmentEntry) -> String {
    egress_entry::primary_egress_score_bucket(item.measured_value())
}
