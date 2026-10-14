use super::*;

use std::collections::BTreeMap;

use crate::overage::overage_rollup as overage_overage_rollup;

pub fn summarize_adaptive_overage_drafts_for_bundle(
    items: &[overage_overage_rollup::AdaptiveOverageDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_limit_bundles_from_draft(items: &[overage_overage_rollup::AdaptiveOverageDraft]) -> Vec<LimitBundle> {
    let mut out: Vec<LimitBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_limit_bundle(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_limit_bundle(item: &LimitBundle) -> String {
    overage_overage_rollup::normalize_manual_overage_rollup_code(&item.code)
}
