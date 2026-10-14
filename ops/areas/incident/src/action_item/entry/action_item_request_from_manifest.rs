use super::*;

use std::collections::BTreeMap;

use crate::action_item::manifest as action_item_manifest;

pub fn summarize_action_item_manifests_for_request(
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

pub fn derive_action_item_requests_from_manifest(
    items: &[action_item_manifest::ActionItemManifest],
) -> Vec<ActionItemRequest> {
    let mut out: Vec<ActionItemRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_action_item_request(item: &ActionItemRequest) -> f64 {
    action_item_manifest::action_item_threshold_weight(6, true)
}
