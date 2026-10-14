use super::*;

use std::collections::BTreeMap;

use crate::alert::reservation as alert_reservation;

pub fn summarize_alert_reservations_for_override(
    items: &[alert_reservation::AlertReservation],
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

pub fn derive_lenient_silence_overrides_from_reservation(
    items: &[alert_reservation::AlertReservation],
) -> Vec<LenientSilenceOverride> {
    let mut out: Vec<LenientSilenceOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_silence_override(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_lenient_silence_override(item: &LenientSilenceOverride) -> String {
    alert_reservation::normalize_alert_agreement_code(&item.code)
}
