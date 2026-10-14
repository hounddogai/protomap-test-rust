use super::*;

use std::collections::BTreeMap;

use crate::query::profile as query_profile;

pub fn summarize_query_tiers_for_summary(items: &[query_profile::QueryTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_fallback_redaction_summaries_from_tier(
    items: &[query_profile::QueryTier],
) -> Vec<FallbackRedactionSummary> {
    let mut out: Vec<FallbackRedactionSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_redaction_summary(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_fallback_redaction_summary(item: &FallbackRedactionSummary) -> String {
    query_profile::normalize_query_snapshot_code(&item.code)
}
