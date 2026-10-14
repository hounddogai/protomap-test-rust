use super::*;

use std::collections::BTreeMap;

use crate::query::profile as query_profile;

pub fn summarize_query_snapshots_for_claim(items: &[query_profile::QuerySnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_retention_claims_from_snapshot(items: &[query_profile::QuerySnapshot]) -> Vec<RetentionClaim> {
    let mut out: Vec<RetentionClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_claim(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_retention_claim(item: &RetentionClaim) -> String {
    query_profile::normalize_regional_query_profile_code(&item.code)
}
