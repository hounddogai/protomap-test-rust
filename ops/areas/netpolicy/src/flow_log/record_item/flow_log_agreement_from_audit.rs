use super::*;

use std::collections::BTreeMap;

use crate::rule::exception as rule_exception;

pub fn summarize_rule_audits_for_agreement(items: &[rule_exception::RuleAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_flow_log_agreements_from_audit(items: &[rule_exception::RuleAudit]) -> Vec<FlowLogAgreement> {
    let mut out: Vec<FlowLogAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_agreement(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_flow_log_agreement(item: &FlowLogAgreement) -> String {
    rule_exception::normalize_rule_exception_code(&item.code)
}
