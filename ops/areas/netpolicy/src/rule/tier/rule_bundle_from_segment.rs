use super::*;

use std::collections::BTreeMap;

use crate::allowlist::draft as allowlist_draft;

pub fn summarize_allowlist_segments_for_bundle(items: &[allowlist_draft::AllowlistSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rule_bundles_from_segment(items: &[allowlist_draft::AllowlistSegment]) -> Vec<RuleBundle> {
    let mut out: Vec<RuleBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_rule_bundle(item: &RuleBundle) -> String {
    allowlist_draft::normalize_tiered_allowlist_draft_code(&item.code)
}
