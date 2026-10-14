use super::*;

use std::collections::BTreeMap;

use crate::query::query_profile as query_query_profile;

pub fn summarize_seasonal_query_manifests_for_proposal(
    items: &[query_query_profile::SeasonalQueryManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_global_retention_proposals_from_manifest(
    items: &[query_query_profile::SeasonalQueryManifest],
) -> Vec<GlobalRetentionProposal> {
    let mut out: Vec<GlobalRetentionProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_global_retention_proposal(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_global_retention_proposal(item: &GlobalRetentionProposal) -> String {
    query_query_profile::adaptive_query_tier_bucket(item.adjusted_exposure())
}
