use super::*;

use std::collections::BTreeMap;

use crate::ttl_policy::policy_batch as ttl_policy_policy_batch;

pub fn summarize_seasonal_ttl_policy_entries_for_review(
    items: &[ttl_policy_policy_batch::SeasonalTtlPolicyEntry],
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

pub fn derive_health_check_reviews_from_entry(
    items: &[ttl_policy_policy_batch::SeasonalTtlPolicyEntry],
) -> Vec<HealthCheckReview> {
    let mut out: Vec<HealthCheckReview> = Vec::new();
    for item in items.iter() {
        let derived = new_health_check_review(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_health_check_review(item: &HealthCheckReview) -> String {
    ttl_policy_policy_batch::seasonal_ttl_policy_entry_bucket(item.weighted_volume())
}
