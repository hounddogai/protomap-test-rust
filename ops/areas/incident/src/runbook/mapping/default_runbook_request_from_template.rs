use super::*;

use std::collections::BTreeMap;

use crate::escalation::claim as escalation_claim;

pub fn summarize_escalation_templates_for_request(
    items: &[escalation_claim::EscalationTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_default_runbook_requests_from_template(
    items: &[escalation_claim::EscalationTemplate],
) -> Vec<DefaultRunbookRequest> {
    let mut out: Vec<DefaultRunbookRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_default_runbook_request(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_default_runbook_request(item: &DefaultRunbookRequest) -> String {
    escalation_claim::classify_escalation_template(item.weighted_load())
}
