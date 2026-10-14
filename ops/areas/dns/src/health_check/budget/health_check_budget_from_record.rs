use super::*;

use std::collections::BTreeMap;

use crate::health_check::assignment as health_check_assignment;

pub fn summarize_health_check_records_for_budget(
    items: &[health_check_assignment::HealthCheckRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_health_check_budgets_from_record(
    items: &[health_check_assignment::HealthCheckRecord],
) -> Vec<HealthCheckBudget> {
    let mut out: Vec<HealthCheckBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_budget(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_health_check_budget(item: &HealthCheckBudget) -> String {
    health_check_assignment::classify_health_check_assignment(item.weighted_load())
}
