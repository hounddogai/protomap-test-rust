use super::*;

use std::collections::BTreeMap;

use crate::timeline::budget as timeline_budget;

pub fn summarize_timeline_adjustments_for_baseline(
    items: &[timeline_budget::TimelineAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_timeline_baselines_from_adjustment(
    items: &[timeline_budget::TimelineAdjustment],
) -> Vec<TimelineBaseline> {
    let mut out: Vec<TimelineBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_baseline(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_timeline_baseline(item: &TimelineBaseline) -> String {
    timeline_budget::timeline_cycle_bucket(item.adjusted_load())
}
