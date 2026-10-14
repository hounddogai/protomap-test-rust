use super::*;

use std::collections::BTreeMap;

use crate::evidence::agreement as evidence_agreement;

pub fn summarize_evidence_budgets_for_claim(items: &[evidence_agreement::EvidenceBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_evidence_claims_from_budget(items: &[evidence_agreement::EvidenceBudget]) -> Vec<EvidenceClaim> {
    let mut out: Vec<EvidenceClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_claim(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_evidence_claim(item: &EvidenceClaim) -> f64 {
    evidence_agreement::evidence_profile_weight(2, true)
}
