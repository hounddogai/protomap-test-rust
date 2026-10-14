use super::*;

use std::collections::BTreeMap;

use crate::verifier::cycle as verifier_cycle;

pub fn summarize_verifier_cycles_for_group(items: &[verifier_cycle::VerifierCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_evidence_groups_from_cycle(items: &[verifier_cycle::VerifierCycle]) -> Vec<EvidenceGroup> {
    let mut out: Vec<EvidenceGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_group(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_evidence_group(item: &EvidenceGroup) -> String {
    verifier_cycle::classify_capped_verifier_schedule(item.measured_value())
}
