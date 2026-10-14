use super::*;

use std::collections::BTreeMap;

use crate::identity::preference as identity_preference;

pub fn summarize_identity_metrics_for_assignment(
    items: &[identity_preference::IdentityMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_credential_assignments_from_metric(
    items: &[identity_preference::IdentityMetric],
) -> Vec<CredentialAssignment> {
    let mut out: Vec<CredentialAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_assignment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_credential_assignment(item: &CredentialAssignment) -> String {
    identity_preference::normalize_identity_bundle_code(&item.code)
}
