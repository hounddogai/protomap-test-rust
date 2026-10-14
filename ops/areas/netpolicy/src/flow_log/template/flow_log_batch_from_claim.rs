use super::*;

use std::collections::BTreeMap;

use crate::rule::claim as rule_claim;

pub fn summarize_primary_rule_claims_for_batch(items: &[rule_claim::PrimaryRuleClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_flow_log_batches_from_claim(items: &[rule_claim::PrimaryRuleClaim]) -> Vec<FlowLogBatch> {
    let mut out: Vec<FlowLogBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_flow_log_batch(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_flow_log_batch(item: &FlowLogBatch) -> String {
    rule_claim::primary_rule_claim_bucket(item.measured_value())
}
