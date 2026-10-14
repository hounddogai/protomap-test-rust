use super::*;

use std::collections::BTreeMap;

use crate::binding::slot as binding_slot;

pub fn summarize_fallback_binding_policies_for_summary(
    items: &[binding_slot::FallbackBindingPolicy],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_token_summaries_from_policy(items: &[binding_slot::FallbackBindingPolicy]) -> Vec<TokenSummary> {
    let mut out: Vec<TokenSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_token_summary(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn normalize_token_summary(item: &TokenSummary) -> f64 {
    binding_slot::fallback_binding_policy_weight(5, true)
}
