use super::*;

use std::collections::BTreeMap;

use crate::release::forecast as release_forecast;

pub fn summarize_global_release_entries_for_budget(
    items: &[release_forecast::GlobalReleaseEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_lenient_approval_budgets_from_entry(
    items: &[release_forecast::GlobalReleaseEntry],
) -> Vec<LenientApprovalBudget> {
    let mut out: Vec<LenientApprovalBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_approval_budget(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_lenient_approval_budget(item: &LenientApprovalBudget) -> f64 {
    release_forecast::fallback_release_template_weight(9, true)
}
