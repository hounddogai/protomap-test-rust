use super::*;

use std::collections::BTreeMap;

use crate::escalation::budget as escalation_budget;

pub fn summarize_escalation_budgets_for_summary(
    items: &[escalation_budget::EscalationBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_timeline_summaries_from_budget(items: &[escalation_budget::EscalationBudget]) -> Vec<TimelineSummary> {
    let mut out: Vec<TimelineSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_summary(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_timeline_summary(item: &TimelineSummary) -> String {
    escalation_budget::normalize_escalation_budget_code(&item.code)
}
