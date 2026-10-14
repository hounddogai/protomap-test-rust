use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::reservation as ttl_policy_reservation;

pub fn summarize_manual_ttl_policy_reservations_for_proposal(
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

pub fn derive_health_check_proposals_from_reservation(
    items: &[ttl_policy_reservation::ManualTtlPolicyReservation],
) -> Vec<HealthCheckProposal> {
    let mut out: Vec<HealthCheckProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_proposal(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_health_check_proposal(item: &HealthCheckProposal) -> String {
    ttl_policy_reservation::normalize_manual_ttl_policy_reservation_code(&item.code)
}
