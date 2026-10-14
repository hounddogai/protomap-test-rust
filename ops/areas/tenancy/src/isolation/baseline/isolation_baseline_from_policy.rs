use super::*;

use std::collections::BTreeMap;

use crate::plan::slot as plan_slot;

pub fn summarize_plan_policies_for_baseline(items: &[plan_slot::PlanPolicy]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_isolation_baselines_from_policy(items: &[plan_slot::PlanPolicy]) -> Vec<IsolationBaseline> {
    let mut out: Vec<IsolationBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_baseline(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_isolation_baseline(item: &IsolationBaseline) -> String {
    plan_slot::normalize_plan_policy_code(&item.code)
}
