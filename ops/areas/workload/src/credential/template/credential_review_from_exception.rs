use super::*;

use std::collections::BTreeMap;

use crate::binding::exception as binding_exception;

pub fn summarize_rolling_binding_exceptions_for_review(
    items: &[binding_exception::RollingBindingException],
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

pub fn derive_credential_reviews_from_exception(
    items: &[binding_exception::RollingBindingException],
) -> Vec<CredentialReview> {
    let mut out: Vec<CredentialReview> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_review(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_credential_review(item: &CredentialReview) -> String {
    binding_exception::rolling_binding_exception_bucket(item.adjusted_exposure())
}
