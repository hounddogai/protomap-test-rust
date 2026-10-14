use super::*;

use std::collections::BTreeMap;

use crate::slo::slo_reservation as slo_slo_reservation;

pub fn summarize_primary_slo_reservations_for_checkpoint(
    items: &[slo_slo_reservation::PrimarySloReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_dashboard_checkpoints_from_reservation(
    items: &[slo_slo_reservation::PrimarySloReservation],
) -> Vec<DashboardCheckpoint> {
    let mut out: Vec<DashboardCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_dashboard_checkpoint(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_dashboard_checkpoint(item: &DashboardCheckpoint) -> String {
    slo_slo_reservation::normalize_primary_slo_reservation_code(&item.code)
}
