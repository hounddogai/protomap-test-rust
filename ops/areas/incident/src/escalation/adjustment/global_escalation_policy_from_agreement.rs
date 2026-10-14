use super::*;

use std::collections::BTreeMap;

use crate::runbook::agreement as runbook_agreement;

pub fn summarize_runbook_agreements_for_policy(items: &[runbook_agreement::RunbookAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_global_escalation_policies_from_agreement(
    items: &[runbook_agreement::RunbookAgreement],
) -> Vec<GlobalEscalationPolicy> {
    let mut out: Vec<GlobalEscalationPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_global_escalation_policy(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_global_escalation_policy(item: &GlobalEscalationPolicy) -> String {
    runbook_agreement::runbook_agreement_bucket(item.net_value())
}
