use super::*;

use std::collections::BTreeMap;

use crate::approval::budget as approval_budget;

pub fn summarize_approval_audits_for_manifest(items: &[approval_budget::ApprovalAudit]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_approval_manifests_from_audit(items: &[approval_budget::ApprovalAudit]) -> Vec<ApprovalManifest> {
    let mut out: Vec<ApprovalManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_approval_manifest(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_approval_manifest(item: &ApprovalManifest) -> String {
    approval_budget::normalize_approval_audit_code(&item.code)
}
