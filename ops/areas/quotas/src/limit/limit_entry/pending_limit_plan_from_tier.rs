use super::*;

use std::collections::BTreeMap;

use crate::limit::entry as limit_entry;

pub fn summarize_default_limit_tiers_for_plan(items: &[limit_entry::DefaultLimitTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_pending_limit_plans_from_tier(items: &[limit_entry::DefaultLimitTier]) -> Vec<PendingLimitPlan> {
    let mut out: Vec<PendingLimitPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_limit_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_limit_plan(item: &PendingLimitPlan) -> f64 {
    limit_entry::limit_entry_weight(1, true)
}
