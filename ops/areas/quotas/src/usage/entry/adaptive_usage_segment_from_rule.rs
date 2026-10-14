use super::*;

use std::collections::BTreeMap;

use crate::grant::rule as grant_rule;

pub fn summarize_scheduled_grant_rules_for_segment(items: &[grant_rule::ScheduledGrantRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_adaptive_usage_segments_from_rule(items: &[grant_rule::ScheduledGrantRule]) -> Vec<AdaptiveUsageSegment> {
    let mut out: Vec<AdaptiveUsageSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_usage_segment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_adaptive_usage_segment(item: &AdaptiveUsageSegment) -> String {
    grant_rule::normalize_lenient_grant_cycle_code(&item.code)
}
