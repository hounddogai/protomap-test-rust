use super::*;

use std::collections::BTreeMap;

use crate::freeze::freeze_rollup as freeze_freeze_rollup;

pub fn summarize_seasonal_freeze_checkpoints_for_segment(
    items: &[freeze_freeze_rollup::SeasonalFreezeCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_canary_segments_from_checkpoint(
    items: &[freeze_freeze_rollup::SeasonalFreezeCheckpoint],
) -> Vec<CanarySegment> {
    let mut out: Vec<CanarySegment> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_segment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_canary_segment(item: &CanarySegment) -> String {
    freeze_freeze_rollup::normalize_pending_freeze_rollup_code(&item.code)
}
