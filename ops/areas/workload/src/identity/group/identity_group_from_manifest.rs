use super::*;

use std::collections::BTreeMap;

use crate::identity::agreement as identity_agreement;

pub fn summarize_global_identity_manifests_for_group(
    items: &[identity_agreement::GlobalIdentityManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_identity_groups_from_manifest(
    items: &[identity_agreement::GlobalIdentityManifest],
) -> Vec<IdentityGroup> {
    let mut out: Vec<IdentityGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_group(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_identity_group(item: &IdentityGroup) -> f64 {
    identity_agreement::identity_agreement_weight(5, true)
}
