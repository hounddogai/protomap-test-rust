use super::*;

use std::collections::BTreeMap;

use crate::identity::milestone as identity_milestone;

pub fn summarize_weighted_identity_preferences_for_draft(
    items: &[identity_milestone::WeightedIdentityPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_lenient_credential_drafts_from_preference(
    items: &[identity_milestone::WeightedIdentityPreference],
) -> Vec<LenientCredentialDraft> {
    let mut out: Vec<LenientCredentialDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_credential_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_lenient_credential_draft(item: &LenientCredentialDraft) -> f64 {
    identity_milestone::identity_draft_weight(3, true)
}
