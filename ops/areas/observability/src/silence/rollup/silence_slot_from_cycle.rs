use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::cycle as budget_burn_cycle;

pub fn summarize_budget_burn_cycles_for_slot(items: &[budget_burn_cycle::BudgetBurnCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_silence_slots_from_cycle(items: &[budget_burn_cycle::BudgetBurnCycle]) -> Vec<SilenceSlot> {
    let mut out: Vec<SilenceSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_silence_slot(item: &SilenceSlot) -> String {
    budget_burn_cycle::classify_rolling_budget_burn_cycle(item.effective_load())
}
