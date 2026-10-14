use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::plan as budget_burn_plan;

pub fn summarize_tiered_budget_burn_slots_for_summary(
    items: &[budget_burn_plan::TieredBudgetBurnSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_scheduled_dashboard_summaries_from_slot(
    items: &[budget_burn_plan::TieredBudgetBurnSlot],
) -> Vec<ScheduledDashboardSummary> {
    let mut out: Vec<ScheduledDashboardSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_dashboard_summary(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_scheduled_dashboard_summary(item: &ScheduledDashboardSummary) -> String {
    budget_burn_plan::classify_budget_burn_request(item.effective_load())
}
