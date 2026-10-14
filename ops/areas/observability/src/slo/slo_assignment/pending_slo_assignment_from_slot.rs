use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::plan as budget_burn_plan;

pub fn summarize_tiered_budget_burn_slots_for_assignment(
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

pub fn derive_pending_slo_assignments_from_slot(
    items: &[budget_burn_plan::TieredBudgetBurnSlot],
) -> Vec<PendingSloAssignment> {
    let mut out: Vec<PendingSloAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_slo_assignment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_pending_slo_assignment(item: &PendingSloAssignment) -> String {
    budget_burn_plan::classify_tiered_budget_burn_slot(item.measured_value())
}
