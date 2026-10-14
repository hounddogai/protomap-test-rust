use super::*;

use std::collections::BTreeMap;

use crate::runbook::variance as runbook_variance;

pub fn summarize_regional_runbook_plans_for_target(
    items: &[runbook_variance::RegionalRunbookPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_pending_action_item_targets_from_plan(
    items: &[runbook_variance::RegionalRunbookPlan],
) -> Vec<PendingActionItemTarget> {
    let mut out: Vec<PendingActionItemTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_action_item_target(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_pending_action_item_target(item: &PendingActionItemTarget) -> String {
    runbook_variance::classify_regional_runbook_plan(item.net_total())
}
