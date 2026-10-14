use super::*;

use std::collections::BTreeMap;

use crate::reservation::item as reservation_item;

pub fn summarize_lenient_reservation_assignments_for_agreement(
    items: &[reservation_item::LenientReservationAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_rolling_grant_agreements_from_assignment(
    items: &[reservation_item::LenientReservationAssignment],
) -> Vec<RollingGrantAgreement> {
    let mut out: Vec<RollingGrantAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_grant_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_grant_agreement(item: &RollingGrantAgreement) -> String {
    reservation_item::normalize_lenient_reservation_assignment_code(&item.code)
}
