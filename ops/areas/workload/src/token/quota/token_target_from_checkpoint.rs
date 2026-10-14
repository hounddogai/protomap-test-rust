use super::*;

use std::collections::BTreeMap;

use crate::identity::manifest as identity_manifest;

pub fn summarize_identity_checkpoints_for_target(
    items: &[identity_manifest::IdentityCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_token_targets_from_checkpoint(items: &[identity_manifest::IdentityCheckpoint]) -> Vec<TokenTarget> {
    let mut out: Vec<TokenTarget> = Vec::new();
    for item in items.iter() {
        let derived = new_token_target(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_token_target(item: &TokenTarget) -> String {
    identity_manifest::normalize_identity_review_code(&item.code)
}
