use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_ledger as credential_credential_ledger;

pub fn summarize_credential_cycles_for_adjustment(
    items: &[credential_credential_ledger::CredentialCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_fallback_binding_adjustments_from_cycle(
    items: &[credential_credential_ledger::CredentialCycle],
) -> Vec<FallbackBindingAdjustment> {
    let mut out: Vec<FallbackBindingAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_binding_adjustment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_fallback_binding_adjustment(item: &FallbackBindingAdjustment) -> String {
    credential_credential_ledger::normalize_credential_cycle_code(&item.code)
}
