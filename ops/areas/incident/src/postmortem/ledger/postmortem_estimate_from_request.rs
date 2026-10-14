use super::*;

use std::collections::BTreeMap;

use crate::escalation::profile as escalation_profile;

pub fn summarize_escalation_requests_for_estimate(
    items: &[escalation_profile::EscalationRequest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_postmortem_estimates_from_request(
    items: &[escalation_profile::EscalationRequest],
) -> Vec<PostmortemEstimate> {
    let mut out: Vec<PostmortemEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_postmortem_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_postmortem_estimate(item: &PostmortemEstimate) -> f64 {
    escalation_profile::escalation_request_weight(9, true)
}
