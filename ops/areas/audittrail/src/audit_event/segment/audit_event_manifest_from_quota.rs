use super::*;

use std::collections::BTreeMap;

use crate::data_export::policy as data_export_policy;

pub fn summarize_data_export_quotas_for_manifest(
    items: &[data_export_policy::DataExportQuota],
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

pub fn derive_audit_event_manifests_from_quota(
    items: &[data_export_policy::DataExportQuota],
) -> Vec<AuditEventManifest> {
    let mut out: Vec<AuditEventManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_manifest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_audit_event_manifest(item: &AuditEventManifest) -> f64 {
    data_export_policy::data_export_quota_weight(6, true)
}
