use super::*;

use std::collections::BTreeMap;

use crate::flow_log::assignment as flow_log_assignment;

pub fn summarize_flow_log_overrides_for_record(
    items: &[flow_log_assignment::FlowLogOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_weighted_rule_records_from_override(
    items: &[flow_log_assignment::FlowLogOverride],
) -> Vec<WeightedRuleRecord> {
    let mut out: Vec<WeightedRuleRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_rule_record(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_weighted_rule_record(item: &WeightedRuleRecord) -> String {
    flow_log_assignment::classify_tiered_flow_log_audit(item.weighted_volume())
}
