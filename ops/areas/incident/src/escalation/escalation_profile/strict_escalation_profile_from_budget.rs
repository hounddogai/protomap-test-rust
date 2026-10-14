use super::*;

use std::collections::BTreeMap;

use crate::page::adjustment as page_adjustment;

pub fn summarize_capped_page_budgets_for_profile(items: &[page_adjustment::CappedPageBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_strict_escalation_profiles_from_budget(
    items: &[page_adjustment::CappedPageBudget],
) -> Vec<StrictEscalationProfile> {
    let mut out: Vec<StrictEscalationProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_escalation_profile(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_strict_escalation_profile(item: &StrictEscalationProfile) -> String {
    page_adjustment::classify_capped_page_budget(item.adjusted_total())
}
