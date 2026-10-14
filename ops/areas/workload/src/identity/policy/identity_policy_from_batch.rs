use super::*;

use std::collections::BTreeMap;

use crate::federation::batch as federation_batch;

pub fn summarize_manual_federation_batches_for_policy(
    items: &[federation_batch::ManualFederationBatch],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_identity_policies_from_batch(items: &[federation_batch::ManualFederationBatch]) -> Vec<IdentityPolicy> {
    let mut out: Vec<IdentityPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_identity_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_identity_policy(item: &IdentityPolicy) -> String {
    federation_batch::normalize_strict_federation_batch_code(&item.code)
}
