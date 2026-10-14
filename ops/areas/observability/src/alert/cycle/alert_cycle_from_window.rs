use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::metric as budget_burn_metric;

pub fn summarize_budget_burn_windows_for_cycle(
    items: &[budget_burn_metric::BudgetBurnWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_alert_cycles_from_window(items: &[budget_burn_metric::BudgetBurnWindow]) -> Vec<AlertCycle> {
    let mut out: Vec<AlertCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_cycle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_alert_cycle(item: &AlertCycle) -> String {
    budget_burn_metric::classify_budget_burn_metric(item.measured_value())
}
