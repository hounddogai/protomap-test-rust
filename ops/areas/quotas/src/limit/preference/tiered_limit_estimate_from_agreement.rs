use super::*;

use std::collections::BTreeMap;

use crate::reservation::proposal as reservation_proposal;

pub fn summarize_reservation_agreements_for_estimate(
    items: &[reservation_proposal::ReservationAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_tiered_limit_estimates_from_agreement(
    items: &[reservation_proposal::ReservationAgreement],
) -> Vec<TieredLimitEstimate> {
    let mut out: Vec<TieredLimitEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_limit_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_tiered_limit_estimate(item: &TieredLimitEstimate) -> String {
    reservation_proposal::weighted_reservation_budget_bucket(item.weighted_total())
}
