use super::*;

use std::collections::BTreeMap;

use crate::approval::variance as approval_variance;

pub fn summarize_approval_plans_for_tier(items: &[approval_variance::ApprovalPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_release_tiers_from_plan(items: &[approval_variance::ApprovalPlan]) -> Vec<ReleaseTier> {
    let mut out: Vec<ReleaseTier> = Vec::new();
    for item in items.iter() {
        let derived = new_release_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_release_tier(item: &ReleaseTier) -> String {
    approval_variance::classify_approval_proposal(item.adjusted_exposure())
}
