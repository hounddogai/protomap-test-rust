use super::*;

use std::collections::BTreeMap;

use crate::identity::schedule as identity_schedule;

pub fn summarize_identity_schedules_for_ledger(items: &[identity_schedule::IdentitySchedule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_batched_credential_ledgers_from_schedule(
    items: &[identity_schedule::IdentitySchedule],
) -> Vec<BatchedCredentialLedger> {
    let mut out: Vec<BatchedCredentialLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_credential_ledger(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_batched_credential_ledger(item: &BatchedCredentialLedger) -> String {
    identity_schedule::normalize_identity_plan_code(&item.code)
}
