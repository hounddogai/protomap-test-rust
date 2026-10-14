use super::*;

use std::collections::BTreeMap;

use crate::alert::budget as alert_budget;

pub fn summarize_alert_budgets_for_profile(items: &[alert_budget::AlertBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_alert_profiles_from_budget(items: &[alert_budget::AlertBudget]) -> Vec<AlertProfile> {
    let mut out: Vec<AlertProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_alert_profile(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_alert_profile(item: &AlertProfile) -> String {
    alert_budget::normalize_rolling_alert_slot_code(&item.code)
}
