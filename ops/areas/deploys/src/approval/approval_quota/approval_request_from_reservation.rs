use super::*;

use std::collections::BTreeMap;

use crate::canary::canary_assignment as canary_canary_assignment;

pub fn summarize_canary_reservations_for_request(
    items: &[canary_canary_assignment::CanaryReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_approval_requests_from_reservation(
    items: &[canary_canary_assignment::CanaryReservation],
) -> Vec<ApprovalRequest> {
    let mut out: Vec<ApprovalRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_approval_request(item: &ApprovalRequest) -> String {
    canary_canary_assignment::normalize_canary_budget_code(&item.code)
}
