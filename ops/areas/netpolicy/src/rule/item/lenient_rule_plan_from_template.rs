use super::*;

use std::collections::BTreeMap;

use crate::flow_log::template as flow_log_template;

pub fn summarize_adaptive_flow_log_templates_for_plan(
    items: &[flow_log_template::AdaptiveFlowLogTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_lenient_rule_plans_from_template(
    items: &[flow_log_template::AdaptiveFlowLogTemplate],
) -> Vec<LenientRulePlan> {
    let mut out: Vec<LenientRulePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_rule_plan(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_lenient_rule_plan(item: &LenientRulePlan) -> String {
    flow_log_template::classify_flow_log_milestone(item.adjusted_load())
}
