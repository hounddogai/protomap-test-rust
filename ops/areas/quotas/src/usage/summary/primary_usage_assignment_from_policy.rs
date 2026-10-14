use super::*;

use std::collections::BTreeMap;

use crate::reservation::policy as reservation_policy;

pub fn summarize_batched_reservation_policies_for_assignment(
    items: &[reservation_policy::BatchedReservationPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_primary_usage_assignments_from_policy(
    items: &[reservation_policy::BatchedReservationPolicy],
) -> Vec<PrimaryUsageAssignment> {
    let mut out: Vec<PrimaryUsageAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_usage_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_primary_usage_assignment(item: &PrimaryUsageAssignment) -> String {
    reservation_policy::normalize_reservation_plan_code(&item.code)
}
