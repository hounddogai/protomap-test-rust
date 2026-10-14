use super::*;

use std::collections::BTreeMap;

use crate::token::threshold as token_threshold;

pub fn summarize_batched_token_ledgers_for_snapshot(
    items: &[token_threshold::BatchedTokenLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_rolling_credential_snapshots_from_ledger(
    items: &[token_threshold::BatchedTokenLedger],
) -> Vec<RollingCredentialSnapshot> {
    let mut out: Vec<RollingCredentialSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_credential_snapshot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_credential_snapshot(item: &RollingCredentialSnapshot) -> String {
    token_threshold::capped_token_schedule_bucket(item.net_exposure())
}
