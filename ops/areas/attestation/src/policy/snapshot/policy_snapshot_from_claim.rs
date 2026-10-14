use super::*;

use std::collections::BTreeMap;

use crate::evidence::claim as evidence_claim;

pub fn summarize_evidence_claims_for_snapshot(items: &[evidence_claim::EvidenceClaim]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_policy_snapshots_from_claim(items: &[evidence_claim::EvidenceClaim]) -> Vec<PolicySnapshot> {
    let mut out: Vec<PolicySnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_snapshot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_policy_snapshot(item: &PolicySnapshot) -> f64 {
    evidence_claim::evidence_claim_weight(4, true)
}
