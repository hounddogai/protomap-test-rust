use super::*;

use std::collections::BTreeMap;

use crate::silence::ledger as silence_ledger;

pub fn summarize_rolling_silence_ledgers_for_estimate(
    items: &[silence_ledger::RollingSilenceLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_dashboard_estimates_from_ledger(
    items: &[silence_ledger::RollingSilenceLedger],
) -> Vec<DashboardEstimate> {
    let mut out: Vec<DashboardEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_estimate(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_dashboard_estimate(item: &DashboardEstimate) -> String {
    silence_ledger::classify_rolling_silence_ledger(item.measured_value())
}
