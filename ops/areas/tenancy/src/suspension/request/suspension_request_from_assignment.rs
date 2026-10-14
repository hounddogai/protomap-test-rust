use super::*;

use std::collections::BTreeMap;

use crate::tenant::tenant_claim as tenant_tenant_claim;

pub fn summarize_tenant_assignments_for_request(
    items: &[tenant_tenant_claim::TenantAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_suspension_requests_from_assignment(
    items: &[tenant_tenant_claim::TenantAssignment],
) -> Vec<SuspensionRequest> {
    let mut out: Vec<SuspensionRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_request(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_suspension_request(item: &SuspensionRequest) -> f64 {
    tenant_tenant_claim::tenant_quota_weight(7, true)
}
