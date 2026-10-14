use super::*;

use std::collections::BTreeMap;

use crate::identity::agreement as identity_agreement;

pub fn summarize_global_identity_manifests_for_entry(
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

pub fn derive_seasonal_credential_entries_from_manifest(
    items: &[identity_agreement::GlobalIdentityManifest],
) -> Vec<SeasonalCredentialEntry> {
    let mut out: Vec<SeasonalCredentialEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_credential_entry(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_seasonal_credential_entry(item: &SeasonalCredentialEntry) -> String {
    identity_agreement::classify_identity_agreement(item.measured_value())
}
