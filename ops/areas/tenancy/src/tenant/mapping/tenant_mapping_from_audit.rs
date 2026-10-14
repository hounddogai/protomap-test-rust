use super::*;

use std::collections::BTreeMap;

use crate::tenant::audit as tenant_audit;

pub fn summarize_fallback_tenant_audits_for_mapping(
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

pub fn derive_tenant_mappings_from_audit(items: &[tenant_audit::FallbackTenantAudit]) -> Vec<TenantMapping> {
    let mut out: Vec<TenantMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_mapping(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_tenant_mapping(item: &TenantMapping) -> f64 {
    tenant_audit::tenant_proposal_weight(9, true)
}
