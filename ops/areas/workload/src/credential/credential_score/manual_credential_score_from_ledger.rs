use super::*;

use std::collections::BTreeMap;

use crate::federation::ledger as federation_ledger;

pub fn summarize_federation_ledgers_for_score(items: &[federation_ledger::FederationLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_manual_credential_scores_from_ledger(
    items: &[federation_ledger::FederationLedger],
) -> Vec<ManualCredentialScore> {
    let mut out: Vec<ManualCredentialScore> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_credential_score(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_manual_credential_score(item: &ManualCredentialScore) -> String {
    federation_ledger::normalize_federation_slot_code(&item.code)
}
