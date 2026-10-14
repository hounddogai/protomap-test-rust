use super::*;

use std::collections::BTreeMap;

use crate::action_item::manifest as action_item_manifest;

pub fn summarize_action_item_manifests_for_variance(
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

pub fn derive_postmortem_variances_from_manifest(
    items: &[action_item_manifest::ActionItemManifest],
) -> Vec<PostmortemVariance> {
    let mut out: Vec<PostmortemVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_variance(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_postmortem_variance(item: &PostmortemVariance) -> String {
    action_item_manifest::normalize_capped_action_item_budget_code(&item.code)
}
