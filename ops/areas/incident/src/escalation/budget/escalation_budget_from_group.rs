use super::*;

use std::collections::BTreeMap;

use crate::page::template as page_template;

pub fn summarize_tiered_page_groups_for_budget(items: &[page_template::TieredPageGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_escalation_budgets_from_group(items: &[page_template::TieredPageGroup]) -> Vec<EscalationBudget> {
    let mut out: Vec<EscalationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_escalation_budget(item: &EscalationBudget) -> String {
    page_template::page_estimate_bucket(item.measured_value())
}
