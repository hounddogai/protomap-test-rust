use super::*;

use std::collections::BTreeMap;

use crate::release::strict_release_policy as release_strict_release_policy;

pub fn summarize_tiered_release_batches_for_segment(
    items: &[release_strict_release_policy::TieredReleaseBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_manual_canary_segments_from_batch(
    items: &[release_strict_release_policy::TieredReleaseBatch],
) -> Vec<ManualCanarySegment> {
    let mut out: Vec<ManualCanarySegment> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_canary_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_manual_canary_segment(item: &ManualCanarySegment) -> String {
    release_strict_release_policy::classify_tiered_release_batch(item.adjusted_exposure())
}
