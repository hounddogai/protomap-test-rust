use super::*;

use std::collections::BTreeMap;

use crate::identity::manifest as identity_manifest;

pub fn summarize_identity_manifests_for_assignment(
    items: &[identity_manifest::IdentityManifest],
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

pub fn derive_binding_assignments_from_manifest(
    items: &[identity_manifest::IdentityManifest],
) -> Vec<BindingAssignment> {
    let mut out: Vec<BindingAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_assignment(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_binding_assignment(item: &BindingAssignment) -> String {
    identity_manifest::identity_review_bucket(item.weighted_value())
}
