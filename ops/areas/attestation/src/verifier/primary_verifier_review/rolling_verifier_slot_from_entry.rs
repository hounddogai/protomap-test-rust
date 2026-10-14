use super::*;

use std::collections::BTreeMap;

use crate::evidence::entry as evidence_entry;

pub fn summarize_evidence_entries_for_slot(items: &[evidence_entry::EvidenceEntry]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_rolling_verifier_slots_from_entry(items: &[evidence_entry::EvidenceEntry]) -> Vec<RollingVerifierSlot> {
    let mut out: Vec<RollingVerifierSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_verifier_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_rolling_verifier_slot(item: &RollingVerifierSlot) -> String {
    evidence_entry::manual_evidence_summary_bucket(item.weighted_value())
}
