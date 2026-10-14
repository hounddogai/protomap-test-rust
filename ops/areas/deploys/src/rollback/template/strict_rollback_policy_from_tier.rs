use super::*;

use std::collections::BTreeMap;

use crate::canary::entry as canary_entry;

pub fn summarize_lenient_canary_tiers_for_policy(items: &[canary_entry::LenientCanaryTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_strict_rollback_policies_from_tier(
    items: &[canary_entry::LenientCanaryTier],
) -> Vec<StrictRollbackPolicy> {
    let mut out: Vec<StrictRollbackPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_rollback_policy(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_strict_rollback_policy(item: &StrictRollbackPolicy) -> String {
    canary_entry::classify_primary_canary_quota(item.measured_value())
}
