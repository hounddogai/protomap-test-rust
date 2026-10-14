use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::exception as trust_domain_exception;

pub fn summarize_trust_domain_exceptions_for_cycle(
    items: &[trust_domain_exception::TrustDomainException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_weighted_federation_cycles_from_exception(
    items: &[trust_domain_exception::TrustDomainException],
) -> Vec<WeightedFederationCycle> {
    let mut out: Vec<WeightedFederationCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_federation_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_weighted_federation_cycle_extra(item: &WeightedFederationCycle) -> String {
    trust_domain_exception::classify_weighted_trust_domain_baseline(item.weighted_exposure())
}
