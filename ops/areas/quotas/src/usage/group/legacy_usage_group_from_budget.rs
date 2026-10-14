use super::*;

use std::collections::BTreeMap;

use crate::reservation::proposal as reservation_proposal;

pub fn summarize_weighted_reservation_budgets_for_group(
    items: &[reservation_proposal::WeightedReservationBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_legacy_usage_groups_from_budget(
    items: &[reservation_proposal::WeightedReservationBudget],
) -> Vec<LegacyUsageGroup> {
    let mut out: Vec<LegacyUsageGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_usage_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_legacy_usage_group(item: &LegacyUsageGroup) -> String {
    reservation_proposal::normalize_reservation_agreement_code(&item.code)
}
