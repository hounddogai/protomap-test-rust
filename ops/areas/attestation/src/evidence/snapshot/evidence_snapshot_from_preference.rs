use super::*;

use std::collections::BTreeMap;

use crate::policy::quota as policy_quota;

pub fn summarize_policy_preferences_for_snapshot(items: &[policy_quota::PolicyPreference]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_evidence_snapshots_from_preference(items: &[policy_quota::PolicyPreference]) -> Vec<EvidenceSnapshot> {
    let mut out: Vec<EvidenceSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_snapshot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_evidence_snapshot(item: &EvidenceSnapshot) -> f64 {
    policy_quota::lenient_policy_metric_weight(5, true)
}
