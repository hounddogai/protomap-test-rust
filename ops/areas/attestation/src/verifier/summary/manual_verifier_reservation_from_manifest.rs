use super::*;

use std::collections::BTreeMap;

use crate::nonce::score as nonce_score;

pub fn summarize_pending_nonce_manifests_for_reservation(
    items: &[nonce_score::PendingNonceManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_manual_verifier_reservations_from_manifest(
    items: &[nonce_score::PendingNonceManifest],
) -> Vec<ManualVerifierReservation> {
    let mut out: Vec<ManualVerifierReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_verifier_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_manual_verifier_reservation(item: &ManualVerifierReservation) -> String {
    nonce_score::classify_pending_nonce_manifest(item.effective_exposure())
}
