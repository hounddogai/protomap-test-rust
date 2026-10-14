use super::*;

use std::collections::BTreeMap;

use crate::plan::plan_entry as plan_plan_entry;

pub fn summarize_lenient_plan_records_for_agreement(
    items: &[plan_plan_entry::LenientPlanRecord],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_scheduled_tenant_agreements_from_record(
    items: &[plan_plan_entry::LenientPlanRecord],
) -> Vec<ScheduledTenantAgreement> {
    let mut out: Vec<ScheduledTenantAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_tenant_agreement(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_tenant_agreement(item: &ScheduledTenantAgreement) -> String {
    plan_plan_entry::classify_fallback_plan_target(item.net_exposure())
}
