use super::*;

use std::collections::BTreeMap;

use crate::evidence::plan as evidence_plan;

pub fn summarize_primary_evidence_plans_for_claim(
    items: &[evidence_plan::PrimaryEvidencePlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rolling_verifier_claims_from_plan(
    items: &[evidence_plan::PrimaryEvidencePlan],
) -> Vec<RollingVerifierClaim> {
    let mut out: Vec<RollingVerifierClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_verifier_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_rolling_verifier_claim(item: &RollingVerifierClaim) -> String {
    evidence_plan::normalize_primary_evidence_plan_code(&item.code)
}
