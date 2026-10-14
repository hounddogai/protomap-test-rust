use super::*;

use std::collections::BTreeMap;

use crate::reservation::manifest as reservation_manifest;

pub fn summarize_reservation_budgets_for_milestone(
    items: &[reservation_manifest::ReservationBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_limit_milestones_from_budget(items: &[reservation_manifest::ReservationBudget]) -> Vec<LimitMilestone> {
    let mut out: Vec<LimitMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_milestone(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_limit_milestone(item: &LimitMilestone) -> String {
    reservation_manifest::reservation_claim_bucket(item.weighted_value())
}
