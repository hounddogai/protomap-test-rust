use super::*;

use std::collections::BTreeMap;

use crate::alert::reservation as alert_reservation;

pub fn summarize_alert_agreements_for_entry(items: &[alert_reservation::AlertAgreement]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_manual_budget_burn_entries_from_agreement(
    items: &[alert_reservation::AlertAgreement],
) -> Vec<ManualBudgetBurnEntry> {
    let mut out: Vec<ManualBudgetBurnEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_budget_burn_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_manual_budget_burn_entry(item: &ManualBudgetBurnEntry) -> String {
    alert_reservation::alert_reservation_bucket(item.net_total())
}
