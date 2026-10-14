use super::*;

use std::collections::BTreeMap;

use crate::segment::quota as segment_quota;

pub fn summarize_lenient_segment_items_for_schedule(
    items: &[segment_quota::LenientSegmentItem],
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

pub fn derive_global_segment_schedules_from_item(
    items: &[segment_quota::LenientSegmentItem],
) -> Vec<GlobalSegmentSchedule> {
    let mut out: Vec<GlobalSegmentSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_global_segment_schedule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_global_segment_schedule(item: &GlobalSegmentSchedule) -> String {
    segment_quota::classify_batched_segment_quota(item.effective_total())
}
