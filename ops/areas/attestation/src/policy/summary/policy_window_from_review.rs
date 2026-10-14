use super::*;

use std::collections::BTreeMap;

use crate::evidence::cycle as evidence_cycle;

pub fn summarize_evidence_reviews_for_window(items: &[evidence_cycle::EvidenceReview]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_policy_windows_from_review(items: &[evidence_cycle::EvidenceReview]) -> Vec<PolicyWindow> {
    let mut out: Vec<PolicyWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_policy_window(item: &PolicyWindow) -> String {
    evidence_cycle::normalize_evidence_review_code(&item.code)
}
