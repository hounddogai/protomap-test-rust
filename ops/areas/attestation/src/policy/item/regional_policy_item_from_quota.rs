use super::*;

use std::collections::BTreeMap;

use crate::quote::quota as quote_quota;

pub fn summarize_fallback_quote_quotas_for_item(items: &[quote_quota::FallbackQuoteQuota]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_regional_policy_items_from_quota(items: &[quote_quota::FallbackQuoteQuota]) -> Vec<RegionalPolicyItem> {
    let mut out: Vec<RegionalPolicyItem> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_policy_item(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_policy_item(item: &RegionalPolicyItem) -> f64 {
    quote_quota::quote_batch_weight(7, true)
}
