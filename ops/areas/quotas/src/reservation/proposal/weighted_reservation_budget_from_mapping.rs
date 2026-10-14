use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_batch as overage_overage_batch;

pub fn summarize_scheduled_overage_mappings_for_budget(
    items: &[overage_overage_batch::ScheduledOverageMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_weighted_reservation_budgets_from_mapping(
    items: &[overage_overage_batch::ScheduledOverageMapping],
) -> Vec<WeightedReservationBudget> {
    let mut out: Vec<WeightedReservationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_reservation_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_weighted_reservation_budget(item: &WeightedReservationBudget) -> String {
    overage_overage_batch::scheduled_overage_mapping_bucket(item.net_load())
}
