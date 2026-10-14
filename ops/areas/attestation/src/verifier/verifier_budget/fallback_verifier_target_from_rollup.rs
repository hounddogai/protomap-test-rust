use super::*;

use std::collections::BTreeMap;

use crate::evidence::milestone as evidence_milestone;

pub fn summarize_evidence_rollups_for_target(items: &[evidence_milestone::EvidenceRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_fallback_verifier_targets_from_rollup(
    items: &[evidence_milestone::EvidenceRollup],
) -> Vec<FallbackVerifierTarget> {
    let mut out: Vec<FallbackVerifierTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_verifier_target(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_fallback_verifier_target(item: &FallbackVerifierTarget) -> String {
    evidence_milestone::classify_evidence_milestone(item.measured_value())
}
