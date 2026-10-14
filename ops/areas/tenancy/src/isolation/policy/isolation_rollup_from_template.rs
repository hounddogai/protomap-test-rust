use super::*;

use std::collections::BTreeMap;

use crate::plan::profile as plan_profile;

pub fn summarize_plan_templates_for_rollup(items: &[plan_profile::PlanTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_isolation_rollups_from_template(items: &[plan_profile::PlanTemplate]) -> Vec<IsolationRollup> {
    let mut out: Vec<IsolationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_rollup(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_isolation_rollup(item: &IsolationRollup) -> String {
    plan_profile::tiered_plan_profile_bucket(item.net_value())
}
