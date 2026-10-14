use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_policy as timeline_timeline_policy;

pub fn summarize_rolling_timeline_policies_for_plan(
    items: &[timeline_timeline_policy::RollingTimelinePolicy],
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

pub fn derive_timeline_plans_from_policy(
    items: &[timeline_timeline_policy::RollingTimelinePolicy],
) -> Vec<TimelinePlan> {
    let mut out: Vec<TimelinePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_plan(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_timeline_plan(item: &TimelinePlan) -> String {
    timeline_timeline_policy::rolling_timeline_policy_bucket(item.net_load())
}
