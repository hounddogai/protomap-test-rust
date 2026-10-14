use super::*;

use std::collections::BTreeMap;

use crate::slo::reservation as slo_reservation;

pub fn summarize_slo_reservations_for_override(items: &[slo_reservation::SloReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_capped_probe_overrides_from_reservation(
    items: &[slo_reservation::SloReservation],
) -> Vec<CappedProbeOverride> {
    let mut out: Vec<CappedProbeOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_probe_override(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_capped_probe_override(item: &CappedProbeOverride) -> f64 {
    slo_reservation::slo_digest_weight(5, true)
}
