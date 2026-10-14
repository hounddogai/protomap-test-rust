use super::*;

use std::collections::BTreeMap;

use crate::approval::threshold as approval_threshold;

pub fn summarize_batched_approval_reservations_for_group(
    items: &[approval_threshold::BatchedApprovalReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_approval_groups_from_reservation(
    items: &[approval_threshold::BatchedApprovalReservation],
) -> Vec<ApprovalGroup> {
    let mut out: Vec<ApprovalGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_group(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_approval_group_extra(item: &ApprovalGroup) -> String {
    approval_threshold::approval_bundle_bucket(item.projected_load())
}
