use super::*;

use std::collections::BTreeMap;

use crate::evidence::bundle as evidence_bundle;

pub fn summarize_evidence_slots_for_baseline(items: &[evidence_bundle::EvidenceSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_rolling_nonce_baselines_from_slot(items: &[evidence_bundle::EvidenceSlot]) -> Vec<RollingNonceBaseline> {
    let mut out: Vec<RollingNonceBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_nonce_baseline(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_rolling_nonce_baseline(item: &RollingNonceBaseline) -> String {
    evidence_bundle::evidence_slot_bucket(item.net_volume())
}
