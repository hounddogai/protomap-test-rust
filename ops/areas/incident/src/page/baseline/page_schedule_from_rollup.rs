use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_rollup as timeline_timeline_rollup;

pub fn summarize_legacy_timeline_rollups_for_schedule(
    items: &[timeline_timeline_rollup::LegacyTimelineRollup],
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

pub fn derive_page_schedules_from_rollup(
    items: &[timeline_timeline_rollup::LegacyTimelineRollup],
) -> Vec<PageSchedule> {
    let mut out: Vec<PageSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_page_schedule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_page_schedule(item: &PageSchedule) -> String {
    timeline_timeline_rollup::classify_timeline_batch(item.measured_value())
}
