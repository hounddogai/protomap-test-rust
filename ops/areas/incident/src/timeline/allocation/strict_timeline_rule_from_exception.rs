use super::*;

use std::collections::BTreeMap;

use crate::timeline::exception as timeline_exception;

pub fn summarize_timeline_exceptions_for_rule(
    items: &[timeline_exception::TimelineException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_strict_timeline_rules_from_exception(
    items: &[timeline_exception::TimelineException],
) -> Vec<StrictTimelineRule> {
    let mut out: Vec<StrictTimelineRule> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_timeline_rule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_strict_timeline_rule(item: &StrictTimelineRule) -> String {
    timeline_exception::classify_timeline_rule(item.measured_value())
}
