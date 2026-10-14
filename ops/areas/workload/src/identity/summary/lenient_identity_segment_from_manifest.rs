use super::*;

use std::collections::BTreeMap;

use crate::credential::credential_score as credential_credential_score;

pub fn summarize_credential_manifests_for_segment(
    items: &[credential_credential_score::CredentialManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_lenient_identity_segments_from_manifest(
    items: &[credential_credential_score::CredentialManifest],
) -> Vec<LenientIdentitySegment> {
    let mut out: Vec<LenientIdentitySegment> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_identity_segment(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_lenient_identity_segment(item: &LenientIdentitySegment) -> String {
    credential_credential_score::credential_manifest_bucket(item.net_load())
}
