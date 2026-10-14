use super::*;

use std::collections::BTreeMap;

use crate::federation::budget as federation_budget;

pub fn summarize_batched_federation_summaries_for_allocation(
    items: &[federation_budget::BatchedFederationSummary],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_primary_trust_domain_allocations_from_summary(
    items: &[federation_budget::BatchedFederationSummary],
) -> Vec<PrimaryTrustDomainAllocation> {
    let mut out: Vec<PrimaryTrustDomainAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_trust_domain_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_primary_trust_domain_allocation(item: &PrimaryTrustDomainAllocation) -> f64 {
    federation_budget::batched_federation_summary_weight(9, true)
}
