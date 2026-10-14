use super::*;

use std::collections::BTreeMap;

use crate::evidence::estimate as evidence_estimate;

pub fn summarize_evidence_estimates_for_entry(items: &[evidence_estimate::EvidenceEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_adaptive_measurement_entries_from_estimate(
    items: &[evidence_estimate::EvidenceEstimate],
) -> Vec<AdaptiveMeasurementEntry> {
    let mut out: Vec<AdaptiveMeasurementEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_measurement_entry(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_adaptive_measurement_entry(item: &AdaptiveMeasurementEntry) -> String {
    evidence_estimate::normalize_evidence_schedule_code(&item.code)
}
