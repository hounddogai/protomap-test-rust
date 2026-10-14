use super::*;

use std::collections::BTreeMap;

use crate::flow_log::record_item as flow_log_record_item;

pub fn summarize_flow_log_records_for_rule(items: &[flow_log_record_item::FlowLogRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_flow_log_rules_from_record(items: &[flow_log_record_item::FlowLogRecord]) -> Vec<FlowLogRule> {
    let mut out: Vec<FlowLogRule> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_rule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_flow_log_rule(item: &FlowLogRule) -> String {
    flow_log_record_item::normalize_flow_log_agreement_code(&item.code)
}
