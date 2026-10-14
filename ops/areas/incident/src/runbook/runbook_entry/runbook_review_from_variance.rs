use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_segment as escalation_escalation_segment;

pub fn summarize_primary_escalation_variances_for_review(
    items: &[escalation_escalation_segment::PrimaryEscalationVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_runbook_reviews_from_variance(
    items: &[escalation_escalation_segment::PrimaryEscalationVariance],
) -> Vec<RunbookReview> {
    let mut out: Vec<RunbookReview> = Vec::new();
    for item in items.iter() {
        let derived = new_runbook_review(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_runbook_review(item: &RunbookReview) -> String {
    escalation_escalation_segment::classify_escalation_score(item.effective_load())
}
