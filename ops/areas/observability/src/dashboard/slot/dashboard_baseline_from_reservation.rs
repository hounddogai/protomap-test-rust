use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::template as budget_burn_template;

pub fn summarize_budget_burn_reservations_for_baseline(
    items: &[budget_burn_template::BudgetBurnReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_dashboard_baselines_from_reservation(
    items: &[budget_burn_template::BudgetBurnReservation],
) -> Vec<DashboardBaseline> {
    let mut out: Vec<DashboardBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_dashboard_baseline(item: &DashboardBaseline) -> String {
    budget_burn_template::regional_budget_burn_template_bucket(item.weighted_volume())
}
