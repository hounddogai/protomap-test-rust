use super::*;

use std::collections::BTreeMap;

use crate::approval::budget as approval_budget;

pub fn summarize_approval_budgets_for_template(items: &[approval_budget::ApprovalBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_release_templates_from_budget(items: &[approval_budget::ApprovalBudget]) -> Vec<ReleaseTemplate> {
    let mut out: Vec<ReleaseTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_release_template(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_release_template(item: &ReleaseTemplate) -> String {
    approval_budget::normalize_approval_baseline_code(&item.code)
}
