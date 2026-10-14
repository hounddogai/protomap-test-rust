use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_claim as tenant_tenant_claim;

pub fn summarize_tenant_quotas_for_schedule(items: &[tenant_tenant_claim::TenantQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_primary_plan_schedules_from_quota(
    items: &[tenant_tenant_claim::TenantQuota],
) -> Vec<PrimaryPlanSchedule> {
    let mut out: Vec<PrimaryPlanSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_plan_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_primary_plan_schedule(item: &PrimaryPlanSchedule) -> String {
    tenant_tenant_claim::classify_tenant_schedule(item.projected_value())
}
