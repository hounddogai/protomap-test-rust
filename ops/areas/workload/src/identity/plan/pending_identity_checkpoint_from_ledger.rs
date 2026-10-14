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

pub fn derive_pending_identity_checkpoints_from_ledger(
    items: &[credential_credential_ledger::BatchedCredentialLedger],
) -> Vec<PendingIdentityCheckpoint> {
    let mut out: Vec<PendingIdentityCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_identity_checkpoint(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_pending_identity_checkpoint(item: &PendingIdentityCheckpoint) -> String {
    credential_credential_ledger::batched_credential_ledger_bucket(item.effective_value())
}
