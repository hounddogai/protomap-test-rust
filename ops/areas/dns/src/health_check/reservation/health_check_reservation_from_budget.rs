use super::*;

use std::collections::BTreeMap;

use crate::health_check::budget as health_check_budget;

pub fn summarize_health_check_budgets_for_reservation(
    items: &[health_check_budget::HealthCheckBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_health_check_reservations_from_budget(
    items: &[health_check_budget::HealthCheckBudget],
) -> Vec<HealthCheckReservation> {
    let mut out: Vec<HealthCheckReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_health_check_reservation(item: &HealthCheckReservation) -> String {
    health_check_budget::health_check_budget_bucket(item.measured_value())
}
