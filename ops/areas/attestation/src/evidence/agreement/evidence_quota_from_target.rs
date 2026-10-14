use super::*;

use std::collections::BTreeMap;

use crate::nonce::variance as nonce_variance;

pub fn summarize_nonce_targets_for_quota(items: &[nonce_variance::NonceTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_evidence_quotas_from_target(items: &[nonce_variance::NonceTarget]) -> Vec<EvidenceQuota> {
    let mut out: Vec<EvidenceQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_quota(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_evidence_quota(item: &EvidenceQuota) -> f64 {
    nonce_variance::nonce_ledger_weight(3, true)
}
