use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_audit as plan_plan_audit;

pub fn summarize_pending_plan_audits_for_quota(items: &[plan_plan_audit::PendingPlanAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_pending_plan_quotas_from_audit(items: &[plan_plan_audit::PendingPlanAudit]) -> Vec<PendingPlanQuota> {
    let mut out: Vec<PendingPlanQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_plan_quota(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_plan_quota(item: &PendingPlanQuota) -> String {
    plan_plan_audit::plan_adjustment_bucket(item.measured_value())
}
