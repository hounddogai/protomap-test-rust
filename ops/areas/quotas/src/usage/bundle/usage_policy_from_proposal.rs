use super::*;

use std::collections::BTreeMap;

use crate::limit::request as limit_request;

pub fn summarize_limit_proposals_for_policy(items: &[limit_request::LimitProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_usage_policies_from_proposal(items: &[limit_request::LimitProposal]) -> Vec<UsagePolicy> {
    let mut out: Vec<UsagePolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_usage_policy(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_usage_policy(item: &UsagePolicy) -> f64 {
    limit_request::limit_audit_weight(3, true)
}
