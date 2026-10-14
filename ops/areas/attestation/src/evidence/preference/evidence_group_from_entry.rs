use super::*;

use std::collections::BTreeMap;

use crate::measurement::entry as measurement_entry;

pub fn summarize_manual_measurement_entries_for_group(
    items: &[measurement_entry::ManualMeasurementEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_evidence_groups_from_entry(items: &[measurement_entry::ManualMeasurementEntry]) -> Vec<EvidenceGroup> {
    let mut out: Vec<EvidenceGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_group(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_evidence_group(item: &EvidenceGroup) -> String {
    measurement_entry::normalize_measurement_agreement_code(&item.code)
}
