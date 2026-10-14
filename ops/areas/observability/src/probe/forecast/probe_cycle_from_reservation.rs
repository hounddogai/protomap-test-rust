use super::*;

use std::collections::BTreeMap;

use crate::silence::quota as silence_quota;

pub fn summarize_silence_reservations_for_cycle(items: &[silence_quota::SilenceReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_probe_cycles_from_reservation(items: &[silence_quota::SilenceReservation]) -> Vec<ProbeCycle> {
    let mut out: Vec<ProbeCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_cycle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_probe_cycle(item: &ProbeCycle) -> f64 {
    silence_quota::silence_quota_weight(8, true)
}
