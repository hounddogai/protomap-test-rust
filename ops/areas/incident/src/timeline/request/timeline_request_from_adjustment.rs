use super::*;

use std::collections::BTreeMap;

use crate::timeline::draft as timeline_draft;

pub fn summarize_manual_timeline_adjustments_for_request(
    items: &[timeline_draft::ManualTimelineAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_timeline_requests_from_adjustment(
    items: &[timeline_draft::ManualTimelineAdjustment],
) -> Vec<TimelineRequest> {
    let mut out: Vec<TimelineRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_request(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_timeline_request(item: &TimelineRequest) -> f64 {
    timeline_draft::manual_timeline_adjustment_weight(2, true)
}
