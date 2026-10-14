use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::reservation as ttl_policy_reservation;

pub fn summarize_manual_ttl_policy_reservations_for_milestone(
    items: &[ttl_policy_reservation::ManualTtlPolicyReservation],
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

pub fn derive_failover_milestones_from_reservation(
    items: &[ttl_policy_reservation::ManualTtlPolicyReservation],
) -> Vec<FailoverMilestone> {
    let mut out: Vec<FailoverMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_milestone(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_failover_milestone(item: &FailoverMilestone) -> String {
    ttl_policy_reservation::classify_batched_ttl_policy_manifest(item.effective_value())
}
