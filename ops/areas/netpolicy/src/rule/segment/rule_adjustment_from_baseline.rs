use super::*;

use std::collections::BTreeMap;

use crate::flow_log::manifest as flow_log_manifest;

pub fn summarize_flow_log_baselines_for_adjustment(
    items: &[flow_log_manifest::FlowLogBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_rule_adjustments_from_baseline(items: &[flow_log_manifest::FlowLogBaseline]) -> Vec<RuleAdjustment> {
    let mut out: Vec<RuleAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_rule_adjustment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_rule_adjustment(item: &RuleAdjustment) -> String {
    flow_log_manifest::flow_log_threshold_bucket(item.net_value())
}
