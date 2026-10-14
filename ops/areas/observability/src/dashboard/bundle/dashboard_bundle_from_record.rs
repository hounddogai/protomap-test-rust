use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::entry as budget_burn_entry;

pub fn summarize_pending_budget_burn_records_for_bundle(
    items: &[budget_burn_entry::PendingBudgetBurnRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_dashboard_bundles_from_record(
    items: &[budget_burn_entry::PendingBudgetBurnRecord],
) -> Vec<DashboardBundle> {
    let mut out: Vec<DashboardBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_bundle(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_dashboard_bundle(item: &DashboardBundle) -> String {
    budget_burn_entry::normalize_strict_budget_burn_template_code(&item.code)
}
