use super::*;

use std::collections::BTreeMap;

use crate::credential::policy as credential_policy;

pub fn summarize_credential_allocations_for_ledger(
    items: &[credential_policy::CredentialAllocation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_binding_ledgers_from_allocation(items: &[credential_policy::CredentialAllocation]) -> Vec<BindingLedger> {
    let mut out: Vec<BindingLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_ledger(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_binding_ledger(item: &BindingLedger) -> f64 {
    credential_policy::credential_allocation_weight(6, true)
}
