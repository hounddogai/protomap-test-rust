use super::*;

use std::collections::BTreeMap;

use crate::verifier::mapping as verifier_mapping;

pub fn summarize_verifier_windows_for_window(items: &[verifier_mapping::VerifierWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_measurement_windows_from_window(items: &[verifier_mapping::VerifierWindow]) -> Vec<MeasurementWindow> {
    let mut out: Vec<MeasurementWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_measurement_window(item: &MeasurementWindow) -> String {
    verifier_mapping::classify_verifier_mapping(item.adjusted_exposure())
}
