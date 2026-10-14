use super::*;

use std::collections::BTreeMap;

use crate::canary::assignment as canary_assignment;

pub fn summarize_canary_requests_for_summary(items: &[canary_assignment::CanaryRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_tiered_canary_summaries_from_request(
    items: &[canary_assignment::CanaryRequest],
) -> Vec<TieredCanarySummary> {
    let mut out: Vec<TieredCanarySummary> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_canary_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_tiered_canary_summary(item: &TieredCanarySummary) -> String {
    canary_assignment::classify_canary_request(item.projected_volume())
}
