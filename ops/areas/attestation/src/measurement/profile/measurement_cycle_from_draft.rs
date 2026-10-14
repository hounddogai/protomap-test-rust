use super::*;

use std::collections::BTreeMap;

use crate::verifier::draft as verifier_draft;

pub fn summarize_verifier_drafts_for_cycle(items: &[verifier_draft::VerifierDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_measurement_cycles_from_draft(items: &[verifier_draft::VerifierDraft]) -> Vec<MeasurementCycle> {
    let mut out: Vec<MeasurementCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_measurement_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_measurement_cycle(item: &MeasurementCycle) -> String {
    verifier_draft::verifier_draft_bucket(item.effective_load())
}
