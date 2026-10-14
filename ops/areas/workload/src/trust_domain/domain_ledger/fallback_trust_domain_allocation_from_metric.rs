use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::assignment as trust_domain_assignment;

pub fn summarize_trust_domain_metrics_for_allocation(
    items: &[trust_domain_assignment::TrustDomainMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_fallback_trust_domain_allocations_from_metric(
    items: &[trust_domain_assignment::TrustDomainMetric],
) -> Vec<FallbackTrustDomainAllocation> {
    let mut out: Vec<FallbackTrustDomainAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_trust_domain_allocation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_fallback_trust_domain_allocation(item: &FallbackTrustDomainAllocation) -> String {
    trust_domain_assignment::classify_regional_trust_domain_notice(item.projected_exposure())
}
