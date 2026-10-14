use super::*;

use std::collections::BTreeMap;

use crate::freeze::seasonal_freeze_reservation as freeze_seasonal_freeze_reservation;

pub fn summarize_freeze_thresholds_for_reservation(
    items: &[freeze_seasonal_freeze_reservation::FreezeThreshold],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_rolling_release_reservations_from_threshold(
    items: &[freeze_seasonal_freeze_reservation::FreezeThreshold],
) -> Vec<RollingReleaseReservation> {
    let mut out: Vec<RollingReleaseReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_release_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_rolling_release_reservation(item: &RollingReleaseReservation) -> f64 {
    freeze_seasonal_freeze_reservation::seasonal_freeze_reservation_weight(1, true)
}
