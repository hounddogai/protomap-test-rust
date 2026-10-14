use super::*;

use std::collections::BTreeMap;

use crate::credential::preference as credential_preference;

pub fn summarize_tiered_credential_rules_for_review(
    items: &[credential_preference::TieredCredentialRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_credential_reviews_from_rule(
    items: &[credential_preference::TieredCredentialRule],
) -> Vec<CredentialReview> {
    let mut out: Vec<CredentialReview> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_review(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_credential_review(item: &CredentialReview) -> String {
    credential_preference::classify_weighted_credential_preference(item.adjusted_exposure())
}
