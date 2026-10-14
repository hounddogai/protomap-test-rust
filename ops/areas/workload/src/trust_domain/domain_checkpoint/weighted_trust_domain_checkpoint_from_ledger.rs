use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_ledger as credential_credential_ledger;

pub fn summarize_batched_credential_ledgers_for_checkpoint(
    items: &[credential_credential_ledger::BatchedCredentialLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_weighted_trust_domain_checkpoints_from_ledger(
    items: &[credential_credential_ledger::BatchedCredentialLedger],
) -> Vec<WeightedTrustDomainCheckpoint> {
    let mut out: Vec<WeightedTrustDomainCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_trust_domain_checkpoint(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_weighted_trust_domain_checkpoint(item: &WeightedTrustDomainCheckpoint) -> String {
    credential_credential_ledger::credential_cycle_bucket(item.adjusted_exposure())
}
