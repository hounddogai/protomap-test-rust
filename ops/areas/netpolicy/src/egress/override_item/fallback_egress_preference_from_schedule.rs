use super::*;

use std::collections::BTreeMap;

use crate::rule::segment as rule_segment;

pub fn summarize_rule_schedules_for_preference(items: &[rule_segment::RuleSchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_fallback_egress_preferences_from_schedule(
    items: &[rule_segment::RuleSchedule],
) -> Vec<FallbackEgressPreference> {
    let mut out: Vec<FallbackEgressPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_egress_preference(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_fallback_egress_preference(item: &FallbackEgressPreference) -> String {
    rule_segment::classify_rule_schedule(item.projected_total())
}
