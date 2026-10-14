use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_bundle as credential_credential_bundle;

pub fn summarize_credential_plans_for_ledger(
    items: &[credential_credential_bundle::CredentialPlan],
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

pub fn derive_batched_federation_ledgers_from_plan(
    items: &[credential_credential_bundle::CredentialPlan],
) -> Vec<BatchedFederationLedger> {
    let mut out: Vec<BatchedFederationLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_federation_ledger(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_batched_federation_ledger(item: &BatchedFederationLedger) -> String {
    credential_credential_bundle::normalize_default_credential_mapping_code(&item.code)
}
