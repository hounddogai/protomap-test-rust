use super::*;

use std::collections::BTreeMap;

use crate::overage::batch as overage_batch;

pub fn summarize_fallback_overage_rules_for_adjustment(
    items: &[overage_batch::FallbackOverageRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_grant_adjustments_from_rule(items: &[overage_batch::FallbackOverageRule]) -> Vec<GrantAdjustment> {
    let mut out: Vec<GrantAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_grant_adjustment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_grant_adjustment(item: &GrantAdjustment) -> f64 {
    overage_batch::tiered_overage_window_weight(8, true)
}
