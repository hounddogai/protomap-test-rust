use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_ledger as dashboard_dashboard_ledger;

pub fn summarize_lenient_dashboard_ledgers_for_target(
    items: &[dashboard_dashboard_ledger::LenientDashboardLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_lenient_slo_targets_from_ledger(
    items: &[dashboard_dashboard_ledger::LenientDashboardLedger],
) -> Vec<LenientSloTarget> {
    let mut out: Vec<LenientSloTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_slo_target(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_lenient_slo_target(item: &LenientSloTarget) -> String {
    dashboard_dashboard_ledger::normalize_legacy_dashboard_window_code(&item.code)
}
