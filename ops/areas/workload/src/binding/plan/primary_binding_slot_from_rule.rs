use super::*;

use std::collections::BTreeMap;

use crate::credential::rule as credential_rule;

pub fn summarize_lenient_credential_rules_for_slot(
    items: &[credential_rule::LenientCredentialRule],
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

pub fn derive_primary_binding_slots_from_rule(
    items: &[credential_rule::LenientCredentialRule],
) -> Vec<PrimaryBindingSlot> {
    let mut out: Vec<PrimaryBindingSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_binding_slot(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_primary_binding_slot(item: &PrimaryBindingSlot) -> String {
    credential_rule::credential_reservation_bucket(item.measured_value())
}
