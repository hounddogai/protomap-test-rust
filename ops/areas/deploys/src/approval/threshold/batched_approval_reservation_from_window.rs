use super::*;

use std::collections::BTreeMap;

use crate::rollback::budget as rollback_budget;

pub fn summarize_rollback_windows_for_reservation(items: &[rollback_budget::RollbackWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_batched_approval_reservations_from_window(
    items: &[rollback_budget::RollbackWindow],
) -> Vec<BatchedApprovalReservation> {
    let mut out: Vec<BatchedApprovalReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_approval_reservation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_batched_approval_reservation(item: &BatchedApprovalReservation) -> f64 {
    rollback_budget::rollback_window_weight(3, true)
}
