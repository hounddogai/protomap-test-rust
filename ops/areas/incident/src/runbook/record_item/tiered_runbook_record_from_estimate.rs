use super::*;

use std::collections::BTreeMap;

use crate::runbook::reservation as runbook_reservation;

pub fn summarize_pending_runbook_estimates_for_record(
    items: &[runbook_reservation::PendingRunbookEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tiered_runbook_records_from_estimate(
    items: &[runbook_reservation::PendingRunbookEstimate],
) -> Vec<TieredRunbookRecord> {
    let mut out: Vec<TieredRunbookRecord> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_runbook_record(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_tiered_runbook_record(item: &TieredRunbookRecord) -> String {
    runbook_reservation::normalize_pending_runbook_estimate_code(&item.code)
}
