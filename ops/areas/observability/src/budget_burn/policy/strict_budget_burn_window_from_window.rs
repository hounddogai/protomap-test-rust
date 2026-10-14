use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_ledger as dashboard_dashboard_ledger;

pub fn summarize_legacy_dashboard_windows_for_window(
    items: &[dashboard_dashboard_ledger::LegacyDashboardWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_strict_budget_burn_windows_from_window(
    items: &[dashboard_dashboard_ledger::LegacyDashboardWindow],
) -> Vec<StrictBudgetBurnWindow> {
    let mut out: Vec<StrictBudgetBurnWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_budget_burn_window(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_budget_burn_window(item: &StrictBudgetBurnWindow) -> String {
    dashboard_dashboard_ledger::dashboard_record_bucket(item.weighted_value())
}
