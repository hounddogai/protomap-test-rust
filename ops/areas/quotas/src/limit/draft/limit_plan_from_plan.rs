use super::*;

use std::collections::BTreeMap;

use crate::usage::bundle as usage_bundle;

pub fn summarize_usage_plans_for_plan(items: &[usage_bundle::UsagePlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_limit_plans_from_plan(items: &[usage_bundle::UsagePlan]) -> Vec<LimitPlan> {
    let mut out: Vec<LimitPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_plan(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_limit_plan(item: &LimitPlan) -> String {
    usage_bundle::classify_usage_policy(item.weighted_value())
}
