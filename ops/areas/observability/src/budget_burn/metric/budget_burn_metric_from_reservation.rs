use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::template as budget_burn_template;

pub fn summarize_budget_burn_reservations_for_metric(
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

pub fn derive_budget_burn_metrics_from_reservation(
    items: &[budget_burn_template::BudgetBurnReservation],
) -> Vec<BudgetBurnMetric> {
    let mut out: Vec<BudgetBurnMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_metric(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_budget_burn_metric(item: &BudgetBurnMetric) -> f64 {
    budget_burn_template::budget_burn_threshold_weight(4, true)
}
