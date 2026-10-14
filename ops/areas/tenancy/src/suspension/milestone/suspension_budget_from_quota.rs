use super::*;

use std::collections::BTreeMap;

use crate::plan::template as plan_template;

pub fn summarize_lenient_plan_quotas_for_budget(items: &[plan_template::LenientPlanQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_suspension_budgets_from_quota(items: &[plan_template::LenientPlanQuota]) -> Vec<SuspensionBudget> {
    let mut out: Vec<SuspensionBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_budget(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_suspension_budget(item: &SuspensionBudget) -> String {
    plan_template::classify_tiered_plan_draft(item.measured_value())
}
