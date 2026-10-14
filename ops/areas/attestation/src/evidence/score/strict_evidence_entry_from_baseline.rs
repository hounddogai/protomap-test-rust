use super::*;

use std::collections::BTreeMap;

use crate::nonce::baseline as nonce_baseline;

pub fn summarize_pending_nonce_baselines_for_entry(
    items: &[nonce_baseline::PendingNonceBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_strict_evidence_entries_from_baseline(
    items: &[nonce_baseline::PendingNonceBaseline],
) -> Vec<StrictEvidenceEntry> {
    let mut out: Vec<StrictEvidenceEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_evidence_entry(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_strict_evidence_entry(item: &StrictEvidenceEntry) -> String {
    nonce_baseline::normalize_nonce_slot_code(&item.code)
}
