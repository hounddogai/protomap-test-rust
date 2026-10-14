use super::*;

use std::collections::BTreeMap;

use crate::flow_log::audit as flow_log_audit;

pub fn summarize_regional_flow_log_segments_for_preference(
    items: &[flow_log_audit::RegionalFlowLogSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_rule_preferences_from_segment(items: &[flow_log_audit::RegionalFlowLogSegment]) -> Vec<RulePreference> {
    let mut out: Vec<RulePreference> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_preference(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_rule_preference(item: &RulePreference) -> String {
    flow_log_audit::classify_regional_flow_log_segment(item.measured_value())
}
