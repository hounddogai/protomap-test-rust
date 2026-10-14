use super::*;

use std::collections::BTreeMap;

use crate::timeline::record_item as timeline_record_item;

pub fn summarize_global_timeline_adjustments_for_rollup(
    items: &[timeline_record_item::GlobalTimelineAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_lenient_timeline_rollups_from_adjustment(
    items: &[timeline_record_item::GlobalTimelineAdjustment],
) -> Vec<LenientTimelineRollup> {
    let mut out: Vec<LenientTimelineRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_timeline_rollup(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_lenient_timeline_rollup(item: &LenientTimelineRollup) -> f64 {
    timeline_record_item::timeline_record_weight(4, true)
}
