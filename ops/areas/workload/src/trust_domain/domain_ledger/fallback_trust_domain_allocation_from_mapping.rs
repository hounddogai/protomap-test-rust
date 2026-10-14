use super::*;

use std::collections::BTreeMap;

use crate::federation::federation_target as federation_federation_target;

pub fn summarize_lenient_federation_mappings_for_allocation(
    items: &[federation_federation_target::LenientFederationMapping],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_fallback_trust_domain_allocations_from_mapping(
    items: &[federation_federation_target::LenientFederationMapping],
) -> Vec<FallbackTrustDomainAllocation> {
    let mut out: Vec<FallbackTrustDomainAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_trust_domain_allocation(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_fallback_trust_domain_allocation(item: &FallbackTrustDomainAllocation) -> String {
    federation_federation_target::lenient_federation_mapping_bucket(item.projected_exposure())
}
