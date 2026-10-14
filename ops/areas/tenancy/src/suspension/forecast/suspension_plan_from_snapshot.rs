use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_review as region_pin_pin_review;

pub fn summarize_region_pin_snapshots_for_plan(
    items: &[region_pin_pin_review::RegionPinSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_suspension_plans_from_snapshot(
    items: &[region_pin_pin_review::RegionPinSnapshot],
) -> Vec<SuspensionPlan> {
    let mut out: Vec<SuspensionPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_suspension_plan(item: &SuspensionPlan) -> f64 {
    region_pin_pin_review::weighted_region_pin_review_weight(4, true)
}
