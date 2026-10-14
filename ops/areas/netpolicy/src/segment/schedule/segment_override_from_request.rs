use super::*;

use std::collections::BTreeMap;

use crate::rule::exception as rule_exception;

pub fn summarize_rule_requests_for_override(items: &[rule_exception::RuleRequest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_segment_overrides_from_request(items: &[rule_exception::RuleRequest]) -> Vec<SegmentOverride> {
    let mut out: Vec<SegmentOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_segment_override(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_segment_override(item: &SegmentOverride) -> String {
    rule_exception::rule_snapshot_bucket(item.net_volume())
}
