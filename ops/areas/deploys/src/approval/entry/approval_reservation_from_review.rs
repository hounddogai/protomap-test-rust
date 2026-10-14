use super::*;

use std::collections::BTreeMap;

use crate::rollout::notice as rollout_notice;

pub fn summarize_pending_rollout_reviews_for_reservation(
    items: &[rollout_notice::PendingRolloutReview],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_approval_reservations_from_review(
    items: &[rollout_notice::PendingRolloutReview],
) -> Vec<ApprovalReservation> {
    let mut out: Vec<ApprovalReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_reservation(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_approval_reservation(item: &ApprovalReservation) -> String {
    rollout_notice::normalize_rollout_notice_code(&item.code)
}
