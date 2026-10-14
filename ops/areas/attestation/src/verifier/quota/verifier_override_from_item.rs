use super::*;

use std::collections::BTreeMap;

use crate::policy::item as policy_item;

pub fn summarize_regional_policy_items_for_override(
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

pub fn derive_verifier_overrides_from_item(items: &[policy_item::RegionalPolicyItem]) -> Vec<VerifierOverride> {
    let mut out: Vec<VerifierOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_verifier_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_verifier_override(item: &VerifierOverride) -> String {
    policy_item::normalize_policy_schedule_code(&item.code)
}
