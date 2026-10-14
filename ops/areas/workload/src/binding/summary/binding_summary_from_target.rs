use super::*;

use std::collections::BTreeMap;

use crate::federation::targets as federation_targets;

pub fn summarize_federation_targets_for_summary(
    items: &[federation_targets::FederationTarget],
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

pub fn derive_binding_summaries_from_target(items: &[federation_targets::FederationTarget]) -> Vec<BindingSummary> {
    let mut out: Vec<BindingSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_binding_summary(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_binding_summary(item: &BindingSummary) -> String {
    federation_targets::normalize_adaptive_federation_cycle_code(&item.code)
}
