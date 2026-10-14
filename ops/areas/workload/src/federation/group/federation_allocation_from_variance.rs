use super::*;

use std::collections::BTreeMap;

use crate::token::variance as token_variance;

pub fn summarize_token_variances_for_allocation(items: &[token_variance::TokenVariance]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_federation_allocations_from_variance(
    items: &[token_variance::TokenVariance],
) -> Vec<FederationAllocation> {
    let mut out: Vec<FederationAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_federation_allocation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_federation_allocation(item: &FederationAllocation) -> String {
    token_variance::token_milestone_bucket(item.effective_exposure())
}
