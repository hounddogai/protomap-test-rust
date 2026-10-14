use super::*;

use std::collections::BTreeMap;

use crate::suspension::suspension_profile as suspension_suspension_profile;

pub fn summarize_batched_suspension_segments_for_preference(
    items: &[suspension_suspension_profile::BatchedSuspensionSegment],
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

pub fn derive_primary_plan_preferences_from_segment(
    items: &[suspension_suspension_profile::BatchedSuspensionSegment],
) -> Vec<PrimaryPlanPreference> {
    let mut out: Vec<PrimaryPlanPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_plan_preference(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_primary_plan_preference(item: &PrimaryPlanPreference) -> String {
    suspension_suspension_profile::classify_tiered_suspension_profile(item.adjusted_total())
}
