use super::*;

use std::collections::BTreeMap;

use crate::health_check::bundle as health_check_bundle;

pub fn summarize_health_check_claims_for_tier(
    items: &[health_check_bundle::HealthCheckClaim],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_ttl_policy_tiers_from_claim(items: &[health_check_bundle::HealthCheckClaim]) -> Vec<TtlPolicyTier> {
    let mut out: Vec<TtlPolicyTier> = Vec::new();
    for item in items.iter() {
        let derived = new_ttl_policy_tier(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_ttl_policy_tier(item: &TtlPolicyTier) -> String {
    health_check_bundle::strict_health_check_score_bucket(item.net_value())
}
