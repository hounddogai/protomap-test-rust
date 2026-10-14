use super::*;

use std::collections::BTreeMap;

use crate::action_item::manifest as action_item_manifest;

pub fn summarize_action_item_manifests_for_estimate(
    items: &[action_item_manifest::ActionItemManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_timeline_estimates_from_manifest(
    items: &[action_item_manifest::ActionItemManifest],
) -> Vec<TimelineEstimate> {
    let mut out: Vec<TimelineEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_estimate(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_timeline_estimate(item: &TimelineEstimate) -> String {
    action_item_manifest::normalize_capped_action_item_budget_code(&item.code)
}
