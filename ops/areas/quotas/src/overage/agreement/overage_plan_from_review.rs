use super::*;

use std::collections::BTreeMap;

use crate::reservation::manifest as reservation_manifest;

pub fn summarize_reservation_reviews_for_plan(
    items: &[reservation_manifest::ReservationReview],
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

pub fn derive_overage_plans_from_review(items: &[reservation_manifest::ReservationReview]) -> Vec<OveragePlan> {
    let mut out: Vec<OveragePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_overage_plan(item: &OveragePlan) -> String {
    reservation_manifest::reservation_claim_bucket(item.projected_total())
}
