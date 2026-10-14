use super::*;

use std::collections::BTreeMap;

use crate::federation::draft as federation_draft;

pub fn summarize_rolling_federation_requests_for_exception(
    items: &[federation_draft::RollingFederationRequest],
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

pub fn derive_binding_exceptions_from_request(
    items: &[federation_draft::RollingFederationRequest],
) -> Vec<BindingException> {
    let mut out: Vec<BindingException> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_exception(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_binding_exception(item: &BindingException) -> String {
    federation_draft::normalize_federation_draft_code(&item.code)
}
