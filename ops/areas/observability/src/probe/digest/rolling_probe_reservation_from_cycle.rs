use super::*;

use std::collections::BTreeMap;

use crate::alert::score as alert_score;

pub fn summarize_batched_alert_cycles_for_reservation(
    items: &[alert_score::BatchedAlertCycle],
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

pub fn derive_rolling_probe_reservations_from_cycle(
    items: &[alert_score::BatchedAlertCycle],
) -> Vec<RollingProbeReservation> {
    let mut out: Vec<RollingProbeReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_probe_reservation(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_rolling_probe_reservation(item: &RollingProbeReservation) -> f64 {
    alert_score::batched_alert_cycle_weight(7, true)
}
