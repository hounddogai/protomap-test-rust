use super::*;

use std::collections::BTreeMap;

use crate::reservation::review as reservation_review;

pub fn summarize_reservation_overrides_for_budget(
    items: &[reservation_review::ReservationOverride],
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

pub fn derive_grant_budgets_from_override(items: &[reservation_review::ReservationOverride]) -> Vec<GrantBudget> {
    let mut out: Vec<GrantBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_grant_budget(item: &GrantBudget) -> String {
    reservation_review::normalize_scheduled_reservation_override_code(&item.code)
}
