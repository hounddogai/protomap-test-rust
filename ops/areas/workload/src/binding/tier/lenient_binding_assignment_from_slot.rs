use super::*;

use std::collections::BTreeMap;

use crate::credential::milestone as credential_milestone;

pub fn summarize_credential_slots_for_assignment(
    items: &[credential_milestone::CredentialSlot],
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

pub fn derive_lenient_binding_assignments_from_slot(
    items: &[credential_milestone::CredentialSlot],
) -> Vec<LenientBindingAssignment> {
    let mut out: Vec<LenientBindingAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_binding_assignment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_binding_assignment(item: &LenientBindingAssignment) -> String {
    credential_milestone::classify_credential_slot(item.effective_load())
}
