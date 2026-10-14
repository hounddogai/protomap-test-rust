use super::*;

use std::collections::BTreeMap;

use crate::verifier::request as verifier_request;

pub fn summarize_manual_verifier_assignments_for_override(
    items: &[verifier_request::ManualVerifierAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_primary_verifier_overrides_from_assignment(
    items: &[verifier_request::ManualVerifierAssignment],
) -> Vec<PrimaryVerifierOverride> {
    let mut out: Vec<PrimaryVerifierOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_verifier_override(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_primary_verifier_override(item: &PrimaryVerifierOverride) -> String {
    verifier_request::classify_fallback_verifier_bundle(item.net_volume())
}
