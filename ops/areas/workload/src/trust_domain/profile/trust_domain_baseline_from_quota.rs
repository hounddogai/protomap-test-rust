use super::*;

use std::collections::BTreeMap;

use crate::federation::federation_proposal as federation_federation_proposal;

pub fn summarize_federation_quotas_for_baseline(
    items: &[federation_federation_proposal::FederationQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_trust_domain_baselines_from_quota(
    items: &[federation_federation_proposal::FederationQuota],
) -> Vec<TrustDomainBaseline> {
    let mut out: Vec<TrustDomainBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_trust_domain_baseline(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_trust_domain_baseline(item: &TrustDomainBaseline) -> String {
    federation_federation_proposal::regional_federation_proposal_bucket(item.weighted_total())
}
