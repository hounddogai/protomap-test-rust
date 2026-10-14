use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_audit as plan_plan_audit;

pub fn summarize_pending_plan_templates_for_schedule(
    items: &[plan_plan_audit::PendingPlanTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_isolation_schedules_from_template(
    items: &[plan_plan_audit::PendingPlanTemplate],
) -> Vec<IsolationSchedule> {
    let mut out: Vec<IsolationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_schedule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_isolation_schedule(item: &IsolationSchedule) -> String {
    plan_plan_audit::classify_pending_plan_audit(item.adjusted_value())
}
