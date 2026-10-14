use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_draft as delegation_delegation_draft;

pub fn summarize_capped_delegation_drafts_for_budget(
    items: &[delegation_delegation_draft::CappedDelegationDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_adaptive_delegation_budgets_from_draft(
    items: &[delegation_delegation_draft::CappedDelegationDraft],
) -> Vec<AdaptiveDelegationBudget> {
    let mut out: Vec<AdaptiveDelegationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_delegation_budget(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_adaptive_delegation_budget(item: &AdaptiveDelegationBudget) -> String {
    delegation_delegation_draft::normalize_delegation_metric_code(&item.code)
}
