use super::*;

use std::collections::BTreeMap;

use crate::runbook::ledger as runbook_ledger;

pub fn summarize_runbook_ledgers_for_group(items: &[runbook_ledger::RunbookLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_primary_escalation_groups_from_ledger(
    items: &[runbook_ledger::RunbookLedger],
) -> Vec<PrimaryEscalationGroup> {
    let mut out: Vec<PrimaryEscalationGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_escalation_group(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_primary_escalation_group(item: &PrimaryEscalationGroup) -> String {
    runbook_ledger::runbook_ledger_bucket(item.net_load())
}
