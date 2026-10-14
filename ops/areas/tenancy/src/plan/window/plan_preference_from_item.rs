use super::*;

use std::collections::BTreeMap;

use crate::plan::schedule as plan_schedule;

pub fn summarize_pending_plan_items_for_preference(items: &[plan_schedule::PendingPlanItem]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_plan_preferences_from_item(items: &[plan_schedule::PendingPlanItem]) -> Vec<PlanPreference> {
    let mut out: Vec<PlanPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_preference(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_plan_preference(item: &PlanPreference) -> f64 {
    plan_schedule::primary_plan_schedule_weight(6, true)
}
