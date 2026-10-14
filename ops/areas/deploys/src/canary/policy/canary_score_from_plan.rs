use super::*;

use std::collections::BTreeMap;

use crate::approval::summary as approval_summary;

pub fn summarize_pending_approval_plans_for_score(
    items: &[approval_summary::PendingApprovalPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_canary_scores_from_plan(items: &[approval_summary::PendingApprovalPlan]) -> Vec<CanaryScore> {
    let mut out: Vec<CanaryScore> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_canary_score(item: &CanaryScore) -> String {
    approval_summary::normalize_approval_summary_code(&item.code)
}
