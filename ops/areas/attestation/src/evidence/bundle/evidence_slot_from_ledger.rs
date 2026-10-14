use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_allocation as nonce_nonce_allocation;

pub fn summarize_fallback_nonce_ledgers_for_slot(
    items: &[nonce_nonce_allocation::FallbackNonceLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_evidence_slots_from_ledger(items: &[nonce_nonce_allocation::FallbackNonceLedger]) -> Vec<EvidenceSlot> {
    let mut out: Vec<EvidenceSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_evidence_slot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_evidence_slot(item: &EvidenceSlot) -> String {
    nonce_nonce_allocation::normalize_lenient_nonce_variance_code(&item.code)
}
