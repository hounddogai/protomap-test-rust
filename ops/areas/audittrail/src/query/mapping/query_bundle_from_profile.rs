use super::*;

use std::collections::BTreeMap;

use crate::redaction::checkpoint as redaction_checkpoint;

pub fn summarize_redaction_profiles_for_bundle(
    items: &[redaction_checkpoint::RedactionProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_query_bundles_from_profile(items: &[redaction_checkpoint::RedactionProfile]) -> Vec<QueryBundle> {
    let mut out: Vec<QueryBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_query_bundle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_query_bundle(item: &QueryBundle) -> String {
    redaction_checkpoint::normalize_pending_redaction_draft_code(&item.code)
}
