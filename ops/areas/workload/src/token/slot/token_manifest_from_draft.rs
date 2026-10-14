use super::*;

use std::collections::BTreeMap;

use crate::federation::draft as federation_draft;

pub fn summarize_federation_drafts_for_manifest(items: &[federation_draft::FederationDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_token_manifests_from_draft(items: &[federation_draft::FederationDraft]) -> Vec<TokenManifest> {
    let mut out: Vec<TokenManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_token_manifest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_token_manifest(item: &TokenManifest) -> String {
    federation_draft::normalize_federation_draft_code(&item.code)
}
