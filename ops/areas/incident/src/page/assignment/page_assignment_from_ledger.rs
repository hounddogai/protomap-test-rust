use super::*;

use std::collections::BTreeMap;

use crate::runbook::ledger as runbook_ledger;

pub fn summarize_runbook_ledgers_for_assignment(items: &[runbook_ledger::RunbookLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_page_assignments_from_ledger(items: &[runbook_ledger::RunbookLedger]) -> Vec<PageAssignment> {
    let mut out: Vec<PageAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_page_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_page_assignment(item: &PageAssignment) -> String {
    runbook_ledger::runbook_ledger_bucket(item.weighted_total())
}
