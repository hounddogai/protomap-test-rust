use super::*;

use std::collections::BTreeMap;

use crate::policy::window as policy_window;

pub fn summarize_manual_policy_quotas_for_manifest(
    items: &[policy_window::ManualPolicyQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_regional_evidence_manifests_from_quota(
    items: &[policy_window::ManualPolicyQuota],
) -> Vec<RegionalEvidenceManifest> {
    let mut out: Vec<RegionalEvidenceManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_evidence_manifest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_regional_evidence_manifest(item: &RegionalEvidenceManifest) -> f64 {
    policy_window::fallback_policy_assignment_weight(7, true)
}
