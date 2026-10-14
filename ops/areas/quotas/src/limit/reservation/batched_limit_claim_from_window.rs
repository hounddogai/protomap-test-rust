use super::*;

use std::collections::BTreeMap;

use crate::grant::digest as grant_digest;

pub fn summarize_grant_windows_for_claim(items: &[grant_digest::GrantWindow]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_batched_limit_claims_from_window(items: &[grant_digest::GrantWindow]) -> Vec<BatchedLimitClaim> {
    let mut out: Vec<BatchedLimitClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_limit_claim(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_batched_limit_claim(item: &BatchedLimitClaim) -> String {
    grant_digest::grant_baseline_bucket(item.adjusted_value())
}
