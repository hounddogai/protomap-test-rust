use super::*;

use std::collections::BTreeMap;

use crate::policy::snapshot as policy_snapshot;

pub fn summarize_policy_snapshots_for_target(items: &[policy_snapshot::PolicySnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_evidence_targets_from_snapshot(items: &[policy_snapshot::PolicySnapshot]) -> Vec<EvidenceTarget> {
    let mut out: Vec<EvidenceTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_target(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_evidence_target(item: &EvidenceTarget) -> String {
    policy_snapshot::normalize_policy_snapshot_code(&item.code)
}
