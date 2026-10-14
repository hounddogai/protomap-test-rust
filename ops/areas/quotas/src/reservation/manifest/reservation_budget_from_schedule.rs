use super::*;

use std::collections::BTreeMap;

use crate::usage::audit as usage_audit;

pub fn summarize_usage_schedules_for_budget(items: &[usage_audit::UsageSchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_reservation_budgets_from_schedule(items: &[usage_audit::UsageSchedule]) -> Vec<ReservationBudget> {
    let mut out: Vec<ReservationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_reservation_budget(item: &ReservationBudget) -> String {
    usage_audit::normalize_rolling_usage_digest_code(&item.code)
}
