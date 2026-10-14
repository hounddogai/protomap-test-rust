use super::*;

use std::collections::BTreeMap;

use crate::timeline::allocation as timeline_allocation;

pub fn summarize_strict_timeline_rules_for_policy(
    items: &[timeline_allocation::StrictTimelineRule],
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

pub fn derive_postmortem_policies_from_rule(
    items: &[timeline_allocation::StrictTimelineRule],
) -> Vec<PostmortemPolicy> {
    let mut out: Vec<PostmortemPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_postmortem_policy(item: &PostmortemPolicy) -> String {
    timeline_allocation::timeline_milestone_bucket(item.effective_total())
}
