use super::*;

use std::collections::BTreeMap;

use crate::flow_log::profile as flow_log_profile;

pub fn summarize_flow_log_profiles_for_entry(items: &[flow_log_profile::FlowLogProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_strict_segment_entries_from_profile(
    items: &[flow_log_profile::FlowLogProfile],
) -> Vec<StrictSegmentEntry> {
    let mut out: Vec<StrictSegmentEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_segment_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_strict_segment_entry(item: &StrictSegmentEntry) -> String {
    flow_log_profile::classify_flow_log_tier(item.measured_value())
}
