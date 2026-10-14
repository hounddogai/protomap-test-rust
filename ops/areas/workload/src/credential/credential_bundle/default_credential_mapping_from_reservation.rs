use super::*;

use std::collections::BTreeMap;

use crate::binding::template as binding_template;

pub fn summarize_lenient_binding_reservations_for_mapping(
    items: &[binding_template::LenientBindingReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_default_credential_mappings_from_reservation(
    items: &[binding_template::LenientBindingReservation],
) -> Vec<DefaultCredentialMapping> {
    let mut out: Vec<DefaultCredentialMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_default_credential_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_default_credential_mapping(item: &DefaultCredentialMapping) -> String {
    binding_template::classify_lenient_binding_reservation(item.net_volume())
}
