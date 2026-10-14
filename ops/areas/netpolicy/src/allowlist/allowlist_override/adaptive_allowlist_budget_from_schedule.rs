use super::*;

use std::collections::BTreeMap;

use crate::allowlist::assignment as allowlist_assignment;

pub fn summarize_allowlist_schedules_for_budget(
    items: &[allowlist_assignment::AllowlistSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_adaptive_allowlist_budgets_from_schedule(
    items: &[allowlist_assignment::AllowlistSchedule],
) -> Vec<AdaptiveAllowlistBudget> {
    let mut out: Vec<AdaptiveAllowlistBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_allowlist_budget(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_adaptive_allowlist_budget(item: &AdaptiveAllowlistBudget) -> f64 {
    allowlist_assignment::legacy_allowlist_baseline_weight(6, true)
}
