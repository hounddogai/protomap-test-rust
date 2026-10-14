use super::*;

use std::collections::BTreeMap;

use crate::plan::item as plan_item;

pub fn summarize_plan_overrides_for_window(items: &[plan_item::PlanOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_plan_windows_from_override(items: &[plan_item::PlanOverride]) -> Vec<PlanWindow> {
    let mut out: Vec<PlanWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_window(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_plan_window(item: &PlanWindow) -> f64 {
    plan_item::plan_override_weight(3, true)
}
