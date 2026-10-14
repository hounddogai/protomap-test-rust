use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::estimate as trust_domain_estimate;

pub fn summarize_fallback_trust_domain_estimates_for_manifest(
    items: &[trust_domain_estimate::FallbackTrustDomainEstimate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_rolling_trust_domain_manifests_from_estimate(
    items: &[trust_domain_estimate::FallbackTrustDomainEstimate],
) -> Vec<RollingTrustDomainManifest> {
    let mut out: Vec<RollingTrustDomainManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_trust_domain_manifest(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn normalize_rolling_trust_domain_manifest(item: &RollingTrustDomainManifest) -> f64 {
    trust_domain_estimate::fallback_trust_domain_estimate_weight(2, true)
}
