use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_summary as canary_canary_summary;

pub fn summarize_primary_canary_summaries_for_snapshot(
    items: &[canary_canary_summary::PrimaryCanarySummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_release_snapshots_from_summary(
    items: &[canary_canary_summary::PrimaryCanarySummary],
) -> Vec<ReleaseSnapshot> {
    let mut out: Vec<ReleaseSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_release_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_release_snapshot_extra(item: &ReleaseSnapshot) -> String {
    canary_canary_summary::primary_canary_summary_bucket(item.adjusted_exposure())
}
