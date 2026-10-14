use super::*;

use std::collections::BTreeMap;

use crate::region_pin::pin_milestone as region_pin_pin_milestone;

pub fn summarize_seasonal_region_pin_milestones_for_schedule(
    items: &[region_pin_pin_milestone::SeasonalRegionPinMilestone],
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

pub fn derive_plan_schedules_from_milestone(
    items: &[region_pin_pin_milestone::SeasonalRegionPinMilestone],
) -> Vec<PlanSchedule> {
    let mut out: Vec<PlanSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_schedule(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_plan_schedule(item: &PlanSchedule) -> String {
    region_pin_pin_milestone::region_pin_metric_bucket(item.weighted_volume())
}
