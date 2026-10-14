use super::*;

use std::collections::BTreeMap;

use crate::nonce::nonce_proposal as nonce_nonce_proposal;

pub fn summarize_legacy_nonce_proposals_for_ledger(
    items: &[nonce_nonce_proposal::LegacyNonceProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_tiered_verifier_ledgers_from_proposal(
    items: &[nonce_nonce_proposal::LegacyNonceProposal],
) -> Vec<TieredVerifierLedger> {
    let mut out: Vec<TieredVerifierLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_verifier_ledger(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_tiered_verifier_ledger(item: &TieredVerifierLedger) -> String {
    nonce_nonce_proposal::default_nonce_target_bucket(item.projected_load())
}
