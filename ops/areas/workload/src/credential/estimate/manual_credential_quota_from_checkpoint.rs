use super::*;

use std::collections::BTreeMap;

use crate::token::token_checkpoint as token_token_checkpoint;

pub fn summarize_capped_token_checkpoints_for_quota(
    items: &[token_token_checkpoint::CappedTokenCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_manual_credential_quotas_from_checkpoint(
    items: &[token_token_checkpoint::CappedTokenCheckpoint],
) -> Vec<ManualCredentialQuota> {
    let mut out: Vec<ManualCredentialQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_credential_quota(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_manual_credential_quota(item: &ManualCredentialQuota) -> String {
    token_token_checkpoint::normalize_batched_token_manifest_code(&item.code)
}
