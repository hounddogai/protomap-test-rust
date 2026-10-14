use super::*;

use std::collections::BTreeMap;

use crate::freeze::metric as freeze_metric;

pub fn summarize_freeze_templates_for_plan(items: &[freeze_metric::FreezeTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_pending_approval_plans_from_template(
    items: &[freeze_metric::FreezeTemplate],
) -> Vec<PendingApprovalPlan> {
    let mut out: Vec<PendingApprovalPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_approval_plan(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_approval_plan(item: &PendingApprovalPlan) -> String {
    freeze_metric::normalize_freeze_metric_code(&item.code)
}
