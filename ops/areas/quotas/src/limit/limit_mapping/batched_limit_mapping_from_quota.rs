use super::*;

use std::collections::BTreeMap;

use crate::grant::quota as grant_quota;

pub fn summarize_grant_quotas_for_mapping(items: &[grant_quota::GrantQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_batched_limit_mappings_from_quota(items: &[grant_quota::GrantQuota]) -> Vec<BatchedLimitMapping> {
    let mut out: Vec<BatchedLimitMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_limit_mapping(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_batched_limit_mapping(item: &BatchedLimitMapping) -> f64 {
    grant_quota::grant_quota_weight(7, true)
}
