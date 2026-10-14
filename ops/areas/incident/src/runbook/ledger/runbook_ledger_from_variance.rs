use super::*;

use std::collections::BTreeMap;

use crate::runbook::runbook_forecast as runbook_runbook_forecast;

pub fn summarize_pending_runbook_variances_for_ledger(
    items: &[runbook_runbook_forecast::PendingRunbookVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_runbook_ledgers_from_variance(
    items: &[runbook_runbook_forecast::PendingRunbookVariance],
) -> Vec<RunbookLedger> {
    let mut out: Vec<RunbookLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_ledger(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_runbook_ledger(item: &RunbookLedger) -> String {
    runbook_runbook_forecast::classify_fallback_runbook_forecast(item.measured_value())
}
