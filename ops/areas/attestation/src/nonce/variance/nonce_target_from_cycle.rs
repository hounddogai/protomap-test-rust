use super::*;

use std::collections::BTreeMap;

use crate::evidence::cycle as evidence_cycle;

pub fn summarize_evidence_cycles_for_target(items: &[evidence_cycle::EvidenceCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_nonce_targets_from_cycle(items: &[evidence_cycle::EvidenceCycle]) -> Vec<NonceTarget> {
    let mut out: Vec<NonceTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_nonce_target(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_nonce_target(item: &NonceTarget) -> String {
    evidence_cycle::normalize_evidence_review_code(&item.code)
}
