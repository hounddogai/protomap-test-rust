use super::*;

use std::collections::BTreeMap;

use crate::health_check::score as health_check_score;

pub fn summarize_primary_health_check_variances_for_bundle(
    items: &[health_check_score::PrimaryHealthCheckVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_weighted_ttl_policy_bundles_from_variance(
    items: &[health_check_score::PrimaryHealthCheckVariance],
) -> Vec<WeightedTtlPolicyBundle> {
    let mut out: Vec<WeightedTtlPolicyBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_ttl_policy_bundle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_weighted_ttl_policy_bundle(item: &WeightedTtlPolicyBundle) -> String {
    health_check_score::classify_manual_health_check_score(item.measured_value())
}
