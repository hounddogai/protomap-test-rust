use super::*;

use std::collections::BTreeMap;

use crate::tenant::manifest as tenant_manifest;

pub fn summarize_lenient_tenant_drafts_for_window(
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

pub fn derive_migration_windows_from_draft(items: &[tenant_manifest::LenientTenantDraft]) -> Vec<MigrationWindow> {
    let mut out: Vec<MigrationWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_migration_window(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_migration_window(item: &MigrationWindow) -> String {
    tenant_manifest::classify_tenant_manifest(item.net_total())
}
