use super::*;

use std::collections::BTreeMap;

use crate::verifier::verifier_budget as verifier_verifier_budget;

pub fn summarize_regional_verifier_rollups_for_plan(
    items: &[verifier_verifier_budget::RegionalVerifierRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_primary_evidence_plans_from_rollup(
    items: &[verifier_verifier_budget::RegionalVerifierRollup],
) -> Vec<PrimaryEvidencePlan> {
    let mut out: Vec<PrimaryEvidencePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_evidence_plan(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_primary_evidence_plan(item: &PrimaryEvidencePlan) -> String {
    verifier_verifier_budget::classify_pending_verifier_assignment(item.adjusted_exposure())
}
