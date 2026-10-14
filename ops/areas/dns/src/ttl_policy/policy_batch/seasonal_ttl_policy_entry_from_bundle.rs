use super::*;

use std::collections::BTreeMap;

use crate::delegation::legacy_delegation_score as delegation_legacy_delegation_score;

pub fn summarize_delegation_bundles_for_entry(
    items: &[delegation_legacy_delegation_score::DelegationBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_seasonal_ttl_policy_entries_from_bundle(
    items: &[delegation_legacy_delegation_score::DelegationBundle],
) -> Vec<SeasonalTtlPolicyEntry> {
    let mut out: Vec<SeasonalTtlPolicyEntry> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_ttl_policy_entry(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn rank_seasonal_ttl_policy_entry(item: &SeasonalTtlPolicyEntry) -> String {
    delegation_legacy_delegation_score::classify_scheduled_delegation_score(item.weighted_volume())
}
