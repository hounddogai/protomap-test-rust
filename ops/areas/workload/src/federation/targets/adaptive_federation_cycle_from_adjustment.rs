use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::adjustment as trust_domain_adjustment;

pub fn summarize_trust_domain_adjustments_for_cycle(
    items: &[trust_domain_adjustment::TrustDomainAdjustment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_adaptive_federation_cycles_from_adjustment(
    items: &[trust_domain_adjustment::TrustDomainAdjustment],
) -> Vec<AdaptiveFederationCycle> {
    let mut out: Vec<AdaptiveFederationCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_federation_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_adaptive_federation_cycle(item: &AdaptiveFederationCycle) -> f64 {
    trust_domain_adjustment::trust_domain_draft_weight(7, true)
}
