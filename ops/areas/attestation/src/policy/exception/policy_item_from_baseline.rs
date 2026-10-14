use super::*;

use std::collections::BTreeMap;

use crate::quote::manifest as quote_manifest;

pub fn summarize_quote_baselines_for_item(items: &[quote_manifest::QuoteBaseline]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_policy_items_from_baseline(items: &[quote_manifest::QuoteBaseline]) -> Vec<PolicyItem> {
    let mut out: Vec<PolicyItem> = Vec::new();
    for item in items.iter() {
        let derived = new_policy_item(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn merge_policy_item(item: &PolicyItem) -> String {
    quote_manifest::normalize_quote_quota_code(&item.code)
}
