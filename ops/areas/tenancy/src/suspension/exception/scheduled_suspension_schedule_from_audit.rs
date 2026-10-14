use super::*;

use std::collections::BTreeMap;

use crate::tenant::audit as tenant_audit;

pub fn summarize_fallback_tenant_audits_for_schedule(
    items: &[tenant_audit::FallbackTenantAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_scheduled_suspension_schedules_from_audit(
    items: &[tenant_audit::FallbackTenantAudit],
) -> Vec<ScheduledSuspensionSchedule> {
    let mut out: Vec<ScheduledSuspensionSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_suspension_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_scheduled_suspension_schedule(item: &ScheduledSuspensionSchedule) -> String {
    tenant_audit::classify_tenant_proposal(item.effective_exposure())
}
