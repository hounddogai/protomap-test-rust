use super::*;

use std::collections::BTreeMap;

use crate::rollback::budget as rollback_budget;

pub fn summarize_rollback_budgets_for_target(items: &[rollback_budget::RollbackBudget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rollback_targets_from_budget(items: &[rollback_budget::RollbackBudget]) -> Vec<RollbackTarget> {
    let mut out: Vec<RollbackTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_rollback_target(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_rollback_target_extra(item: &RollbackTarget) -> String {
    rollback_budget::classify_rollback_budget(item.weighted_volume())
}
