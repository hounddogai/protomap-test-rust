use super::*;

use std::collections::BTreeMap;

use crate::tenant::batched_tenant_audit as tenant_batched_tenant_audit;

pub fn summarize_tenant_profiles_for_proposal(
    items: &[tenant_batched_tenant_audit::TenantProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_pending_region_pin_proposals_from_profile(
    items: &[tenant_batched_tenant_audit::TenantProfile],
) -> Vec<PendingRegionPinProposal> {
    let mut out: Vec<PendingRegionPinProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_region_pin_proposal(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_pending_region_pin_proposal(item: &PendingRegionPinProposal) -> String {
    tenant_batched_tenant_audit::normalize_legacy_tenant_audit_code(&item.code)
}
