use super::*;

use std::collections::BTreeMap;

use crate::plan::estimate as plan_estimate;

pub fn summarize_plan_estimates_for_baseline(items: &[plan_estimate::PlanEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_plan_baselines_from_estimate(items: &[plan_estimate::PlanEstimate]) -> Vec<PlanBaseline> {
    let mut out: Vec<PlanBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_plan_baseline(item: &PlanBaseline) -> String {
    plan_estimate::normalize_plan_profile_code(&item.code)
}
