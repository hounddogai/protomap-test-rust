use super::*;

use std::collections::BTreeMap;

use crate::credential::claim as credential_claim;

pub fn summarize_lenient_credential_segments_for_budget(
    items: &[credential_claim::LenientCredentialSegment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_federation_budgets_from_segment(
    items: &[credential_claim::LenientCredentialSegment],
) -> Vec<FederationBudget> {
    let mut out: Vec<FederationBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_budget(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_federation_budget(item: &FederationBudget) -> String {
    credential_claim::normalize_credential_claim_code(&item.code)
}
