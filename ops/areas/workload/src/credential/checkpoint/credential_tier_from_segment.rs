use super::*;

use std::collections::BTreeMap;

use crate::credential::summary as credential_summary;

pub fn summarize_strict_credential_segments_for_tier(
    items: &[credential_summary::StrictCredentialSegment],
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

pub fn derive_credential_tiers_from_segment(
    items: &[credential_summary::StrictCredentialSegment],
) -> Vec<CredentialTier> {
    let mut out: Vec<CredentialTier> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_credential_tier(item: &CredentialTier) -> String {
    credential_summary::normalize_tiered_credential_policy_code(&item.code)
}
