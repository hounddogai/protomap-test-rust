use super::*;

use std::collections::BTreeMap;

use crate::region_pin::legacy_pin_override as region_pin_legacy_pin_override;

pub fn summarize_region_pin_budgets_for_override(
    items: &[region_pin_legacy_pin_override::RegionPinBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_capped_suspension_overrides_from_budget(
    items: &[region_pin_legacy_pin_override::RegionPinBudget],
) -> Vec<CappedSuspensionOverride> {
    let mut out: Vec<CappedSuspensionOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_suspension_override(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_capped_suspension_override(item: &CappedSuspensionOverride) -> f64 {
    region_pin_legacy_pin_override::region_pin_window_weight(4, true)
}
