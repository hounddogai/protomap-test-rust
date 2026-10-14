use super::*;

use std::collections::BTreeMap;

use crate::reservation::exception as reservation_exception;

pub fn summarize_primary_reservation_templates_for_plan(
    items: &[reservation_exception::PrimaryReservationTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_global_usage_plans_from_template(
    items: &[reservation_exception::PrimaryReservationTemplate],
) -> Vec<GlobalUsagePlan> {
    let mut out: Vec<GlobalUsagePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_global_usage_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_global_usage_plan(item: &GlobalUsagePlan) -> String {
    reservation_exception::normalize_primary_reservation_template_code(&item.code)
}
