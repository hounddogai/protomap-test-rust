use super::*;

use std::collections::BTreeMap;

use crate::policy::item as policy_item;

pub fn summarize_regional_policy_items_for_baseline(
    items: &[policy_item::RegionalPolicyItem],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_strict_quote_baselines_from_item(items: &[policy_item::RegionalPolicyItem]) -> Vec<StrictQuoteBaseline> {
    let mut out: Vec<StrictQuoteBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_quote_baseline(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_strict_quote_baseline(item: &StrictQuoteBaseline) -> String {
    policy_item::normalize_policy_schedule_code(&item.code)
}
