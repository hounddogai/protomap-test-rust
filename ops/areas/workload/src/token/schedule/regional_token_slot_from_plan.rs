use super::*;

use std::collections::BTreeMap;

use crate::token::batch as token_batch;

pub fn summarize_weighted_token_plans_for_slot(items: &[token_batch::WeightedTokenPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_regional_token_slots_from_plan(items: &[token_batch::WeightedTokenPlan]) -> Vec<RegionalTokenSlot> {
    let mut out: Vec<RegionalTokenSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_token_slot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_regional_token_slot(item: &RegionalTokenSlot) -> String {
    token_batch::classify_weighted_token_plan(item.projected_value())
}
