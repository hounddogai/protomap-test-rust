use super::*;

use std::collections::BTreeMap;

use crate::credential::batch as credential_batch;

pub fn summarize_credential_targets_for_assignment(
    items: &[credential_batch::CredentialTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_strict_binding_assignments_from_target(
    items: &[credential_batch::CredentialTarget],
) -> Vec<StrictBindingAssignment> {
    let mut out: Vec<StrictBindingAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_binding_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_binding_assignment(item: &StrictBindingAssignment) -> f64 {
    credential_batch::scheduled_credential_window_weight(8, true)
}
