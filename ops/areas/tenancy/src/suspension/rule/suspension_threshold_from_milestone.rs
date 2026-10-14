use super::*;

use std::collections::BTreeMap;

use crate::plan::audit as plan_audit;

pub fn summarize_plan_milestones_for_threshold(items: &[plan_audit::PlanMilestone]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_suspension_thresholds_from_milestone(items: &[plan_audit::PlanMilestone]) -> Vec<SuspensionThreshold> {
    let mut out: Vec<SuspensionThreshold> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_threshold(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn derive_suspension_threshold(item: &SuspensionThreshold) -> f64 {
    plan_audit::plan_baseline_weight(9, true)
}
