use super::*;

use std::collections::BTreeMap;

use crate::segment::notice as segment_notice;

pub fn summarize_default_segment_notices_for_adjustment(
    items: &[segment_notice::DefaultSegmentNotice],
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

pub fn derive_legacy_allowlist_adjustments_from_notice(
    items: &[segment_notice::DefaultSegmentNotice],
) -> Vec<LegacyAllowlistAdjustment> {
    let mut out: Vec<LegacyAllowlistAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_allowlist_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_legacy_allowlist_adjustment(item: &LegacyAllowlistAdjustment) -> f64 {
    segment_notice::segment_mapping_weight(8, true)
}
