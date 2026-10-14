use super::*;

use std::collections::BTreeMap;

use crate::release::strict_release_policy as release_strict_release_policy;

pub fn summarize_default_release_reviews_for_reservation(
    items: &[release_strict_release_policy::DefaultReleaseReview],
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

pub fn derive_regional_approval_reservations_from_review(
    items: &[release_strict_release_policy::DefaultReleaseReview],
) -> Vec<RegionalApprovalReservation> {
    let mut out: Vec<RegionalApprovalReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_approval_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_regional_approval_reservation(item: &RegionalApprovalReservation) -> String {
    release_strict_release_policy::global_release_policy_bucket(item.projected_total())
}
