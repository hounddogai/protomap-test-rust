use super::*;

use std::collections::BTreeMap;

use crate::identity::targets as identity_targets;

pub fn summarize_adaptive_identity_targets_for_score(
    items: &[identity_targets::AdaptiveIdentityTarget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_capped_trust_domain_scores_from_target(
    items: &[identity_targets::AdaptiveIdentityTarget],
) -> Vec<CappedTrustDomainScore> {
    let mut out: Vec<CappedTrustDomainScore> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_trust_domain_score(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_capped_trust_domain_score(item: &CappedTrustDomainScore) -> String {
    identity_targets::pending_identity_summary_bucket(item.net_total())
}
