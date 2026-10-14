use super::*;

use std::collections::BTreeMap;

use crate::runbook::bundle as runbook_bundle;

pub fn summarize_batched_runbook_bundles_for_milestone(
    items: &[runbook_bundle::BatchedRunbookBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_postmortem_milestones_from_bundle(
    items: &[runbook_bundle::BatchedRunbookBundle],
) -> Vec<PostmortemMilestone> {
    let mut out: Vec<PostmortemMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_milestone(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_postmortem_milestone(item: &PostmortemMilestone) -> String {
    runbook_bundle::normalize_batched_runbook_bundle_code(&item.code)
}
