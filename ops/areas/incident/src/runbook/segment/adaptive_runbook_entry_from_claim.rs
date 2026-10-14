use super::*;

use std::collections::BTreeMap;

use crate::timeline::claim as timeline_claim;

pub fn summarize_timeline_claims_for_entry(items: &[timeline_claim::TimelineClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_adaptive_runbook_entries_from_claim(
    items: &[timeline_claim::TimelineClaim],
) -> Vec<AdaptiveRunbookEntry> {
    let mut out: Vec<AdaptiveRunbookEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_runbook_entry(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_adaptive_runbook_entry(item: &AdaptiveRunbookEntry) -> String {
    timeline_claim::classify_weighted_timeline_plan(item.adjusted_volume())
}
