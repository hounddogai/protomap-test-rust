use super::*;

use std::collections::BTreeMap;

use crate::limit::request as limit_request;

pub fn summarize_limit_proposals_for_milestone(items: &[limit_request::LimitProposal]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_overage_milestones_from_proposal(items: &[limit_request::LimitProposal]) -> Vec<OverageMilestone> {
    let mut out: Vec<OverageMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_milestone(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_overage_milestone(item: &OverageMilestone) -> String {
    limit_request::normalize_limit_draft_code(&item.code)
}
