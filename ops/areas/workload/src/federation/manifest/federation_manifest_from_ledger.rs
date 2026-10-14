use super::*;

use std::collections::BTreeMap;

use crate::credential::snapshot as credential_snapshot;

pub fn summarize_fallback_credential_ledgers_for_manifest(
    items: &[credential_snapshot::FallbackCredentialLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_federation_manifests_from_ledger(
    items: &[credential_snapshot::FallbackCredentialLedger],
) -> Vec<FederationManifest> {
    let mut out: Vec<FederationManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_federation_manifest(item: &FederationManifest) -> f64 {
    credential_snapshot::rolling_credential_snapshot_weight(7, true)
}
