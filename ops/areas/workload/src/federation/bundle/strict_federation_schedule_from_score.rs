use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::targets as trust_domain_targets;

pub fn summarize_lenient_trust_domain_scores_for_schedule(
    items: &[trust_domain_targets::LenientTrustDomainScore],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_strict_federation_schedules_from_score(
    items: &[trust_domain_targets::LenientTrustDomainScore],
) -> Vec<StrictFederationSchedule> {
    let mut out: Vec<StrictFederationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_federation_schedule(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_strict_federation_schedule(item: &StrictFederationSchedule) -> f64 {
    trust_domain_targets::lenient_trust_domain_score_weight(8, true)
}
