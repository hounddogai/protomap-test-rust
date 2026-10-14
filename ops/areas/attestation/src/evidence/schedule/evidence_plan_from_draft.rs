use super::*;

use std::collections::BTreeMap;

use crate::evidence::estimate as evidence_estimate;

pub fn summarize_manual_evidence_drafts_for_plan(
    items: &[evidence_estimate::ManualEvidenceDraft],
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

pub fn derive_evidence_plans_from_draft(items: &[evidence_estimate::ManualEvidenceDraft]) -> Vec<EvidencePlan> {
    let mut out: Vec<EvidencePlan> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_plan(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_evidence_plan(item: &EvidencePlan) -> String {
    evidence_estimate::classify_seasonal_evidence_score(item.measured_value())
}
