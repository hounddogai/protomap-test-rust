use super::*;

use std::collections::BTreeMap;

use crate::federation::group as federation_group;

pub fn summarize_federation_groups_for_summary(items: &[federation_group::FederationGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_credential_summaries_from_group(items: &[federation_group::FederationGroup]) -> Vec<CredentialSummary> {
    let mut out: Vec<CredentialSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_credential_summary(item: &CredentialSummary) -> String {
    federation_group::normalize_federation_group_code(&item.code)
}
