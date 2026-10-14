use super::*;

use std::collections::BTreeMap;

use crate::freeze::seasonal_freeze_reservation as freeze_seasonal_freeze_reservation;

pub fn summarize_seasonal_freeze_reservations_for_threshold(
    items: &[freeze_seasonal_freeze_reservation::SeasonalFreezeReservation],
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

pub fn derive_capped_canary_thresholds_from_reservation(
    items: &[freeze_seasonal_freeze_reservation::SeasonalFreezeReservation],
) -> Vec<CappedCanaryThreshold> {
    let mut out: Vec<CappedCanaryThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_canary_threshold(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_capped_canary_threshold(item: &CappedCanaryThreshold) -> f64 {
    freeze_seasonal_freeze_reservation::freeze_threshold_weight(1, true)
}
