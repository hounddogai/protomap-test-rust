use super::*;

use std::collections::BTreeMap;

use crate::rule::budget as rule_budget;

pub fn summarize_rule_templates_for_quota(items: &[rule_budget::RuleTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_flow_log_quotas_from_template(items: &[rule_budget::RuleTemplate]) -> Vec<FlowLogQuota> {
    let mut out: Vec<FlowLogQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_flow_log_quota(item: &FlowLogQuota) -> String {
    rule_budget::normalize_regional_rule_tier_code(&item.code)
}
