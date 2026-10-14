use super::*;

use std::collections::BTreeMap;

use crate::page::profile as page_profile;

pub fn summarize_page_mappings_for_manifest(items: &[page_profile::PageMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_action_item_manifests_from_mapping(items: &[page_profile::PageMapping]) -> Vec<ActionItemManifest> {
    let mut out: Vec<ActionItemManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_manifest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_action_item_manifest(item: &ActionItemManifest) -> String {
    page_profile::classify_page_mapping(item.adjusted_total())
}
