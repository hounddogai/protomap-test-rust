use super::*;

use std::collections::BTreeMap;

use crate::identity::tier as identity_tier;

pub fn summarize_identity_quotas_for_variance(items: &[identity_tier::IdentityQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_token_variances_from_quota(items: &[identity_tier::IdentityQuota]) -> Vec<TokenVariance> {
    let mut out: Vec<TokenVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_token_variance(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_token_variance(item: &TokenVariance) -> String {
    identity_tier::identity_tier_bucket(item.measured_value())
}
