use super::*;

use std::collections::BTreeMap;

use crate::runbook::exception as runbook_exception;

pub fn summarize_runbook_groups_for_claim(items: &[runbook_exception::RunbookGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_pending_escalation_claims_from_group(
    items: &[runbook_exception::RunbookGroup],
) -> Vec<PendingEscalationClaim> {
    let mut out: Vec<PendingEscalationClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_escalation_claim(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_escalation_claim(item: &PendingEscalationClaim) -> String {
    runbook_exception::pending_runbook_exception_bucket(item.effective_exposure())
}
