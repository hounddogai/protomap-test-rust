use super::*;

use std::collections::BTreeMap;

use crate::suspension::manifest as suspension_manifest;

pub fn summarize_regional_suspension_profiles_for_audit(
    items: &[suspension_manifest::RegionalSuspensionProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_legacy_tenant_audits_from_profile(
    items: &[suspension_manifest::RegionalSuspensionProfile],
) -> Vec<LegacyTenantAudit> {
    let mut out: Vec<LegacyTenantAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_tenant_audit(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_legacy_tenant_audit(item: &LegacyTenantAudit) -> f64 {
    suspension_manifest::scheduled_suspension_manifest_weight(3, true)
}
