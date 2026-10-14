use super::*;

use std::collections::BTreeMap;

use crate::tenant::manifest as tenant_manifest;

pub fn summarize_lenient_tenant_drafts_for_assignment(
    items: &[tenant_manifest::LenientTenantDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_tenant_assignments_from_draft(items: &[tenant_manifest::LenientTenantDraft]) -> Vec<TenantAssignment> {
    let mut out: Vec<TenantAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_tenant_assignment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_tenant_assignment(item: &TenantAssignment) -> f64 {
    tenant_manifest::lenient_tenant_draft_weight(2, true)
}
