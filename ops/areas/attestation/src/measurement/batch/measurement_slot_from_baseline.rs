use super::*;

use std::collections::BTreeMap;

use crate::verifier::segment as verifier_segment;

pub fn summarize_verifier_baselines_for_slot(items: &[verifier_segment::VerifierBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_measurement_slots_from_baseline(items: &[verifier_segment::VerifierBaseline]) -> Vec<MeasurementSlot> {
    let mut out: Vec<MeasurementSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_slot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_measurement_slot(item: &MeasurementSlot) -> String {
    verifier_segment::verifier_baseline_bucket(item.net_value())
}
