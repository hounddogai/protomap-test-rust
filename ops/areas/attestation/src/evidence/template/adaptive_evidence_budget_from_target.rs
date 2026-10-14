use super::*;

use std::collections::BTreeMap;

use crate::evidence::assignment as evidence_assignment;

pub fn summarize_evidence_targets_for_budget(items: &[evidence_assignment::EvidenceTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_adaptive_evidence_budgets_from_target(
    items: &[evidence_assignment::EvidenceTarget],
) -> Vec<AdaptiveEvidenceBudget> {
    let mut out: Vec<AdaptiveEvidenceBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_evidence_budget(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_adaptive_evidence_budget(item: &AdaptiveEvidenceBudget) -> String {
    evidence_assignment::normalize_evidence_exception_code(&item.code)
}
