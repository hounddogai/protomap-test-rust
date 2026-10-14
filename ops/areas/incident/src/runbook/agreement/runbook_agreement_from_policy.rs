use super::*;

use std::collections::BTreeMap;

use crate::escalation::policy as escalation_policy;

pub fn summarize_escalation_policies_for_agreement(
    items: &[escalation_policy::EscalationPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_runbook_agreements_from_policy(items: &[escalation_policy::EscalationPolicy]) -> Vec<RunbookAgreement> {
    let mut out: Vec<RunbookAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_agreement(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_runbook_agreement(item: &RunbookAgreement) -> String {
    escalation_policy::normalize_escalation_metric_code(&item.code)
}
