use super::*;

use std::collections::BTreeMap;

use crate::allowlist::targets as allowlist_targets;

pub fn summarize_manual_allowlist_targets_for_baseline(
    items: &[allowlist_targets::ManualAllowlistTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_segment_baselines_from_target(
    items: &[allowlist_targets::ManualAllowlistTarget],
) -> Vec<ScheduledSegmentBaseline> {
    let mut out: Vec<ScheduledSegmentBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_segment_baseline(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_scheduled_segment_baseline(item: &ScheduledSegmentBaseline) -> String {
    allowlist_targets::classify_manual_allowlist_target(item.projected_exposure())
}
