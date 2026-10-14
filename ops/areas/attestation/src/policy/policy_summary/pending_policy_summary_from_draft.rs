use super::*;

use std::collections::BTreeMap;

use crate::verifier::draft as verifier_draft;

pub fn summarize_verifier_drafts_for_summary(items: &[verifier_draft::VerifierDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_pending_policy_summaries_from_draft(
    items: &[verifier_draft::VerifierDraft],
) -> Vec<PendingPolicySummary> {
    let mut out: Vec<PendingPolicySummary> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_policy_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_pending_policy_summary(item: &PendingPolicySummary) -> String {
    verifier_draft::classify_verifier_draft(item.effective_value())
}
