use super::*;

use std::collections::BTreeMap;

use crate::escalation::escalation_rollup as escalation_escalation_rollup;

pub fn summarize_tiered_escalation_rollups_for_budget(
    items: &[escalation_escalation_rollup::TieredEscalationRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_page_budgets_from_rollup(
    items: &[escalation_escalation_rollup::TieredEscalationRollup],
) -> Vec<PageBudget> {
    let mut out: Vec<PageBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_page_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_page_budget(item: &PageBudget) -> String {
    escalation_escalation_rollup::classify_batched_escalation_adjustment(item.adjusted_exposure())
}
