use super::*;

use std::collections::BTreeMap;

use crate::allowlist::draft as allowlist_draft;

pub fn summarize_allowlist_segments_for_rollup(items: &[allowlist_draft::AllowlistSegment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_default_egress_rollups_from_segment(
    items: &[allowlist_draft::AllowlistSegment],
) -> Vec<DefaultEgressRollup> {
    let mut out: Vec<DefaultEgressRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_default_egress_rollup(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_default_egress_rollup(item: &DefaultEgressRollup) -> String {
    allowlist_draft::tiered_allowlist_draft_bucket(item.adjusted_volume())
}
