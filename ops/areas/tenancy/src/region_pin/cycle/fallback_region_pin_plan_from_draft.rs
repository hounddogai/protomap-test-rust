use super::*;

use std::collections::BTreeMap;

use crate::region_pin::draft as region_pin_draft;

pub fn summarize_region_pin_drafts_for_plan(items: &[region_pin_draft::RegionPinDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_fallback_region_pin_plans_from_draft(
    items: &[region_pin_draft::RegionPinDraft],
) -> Vec<FallbackRegionPinPlan> {
    let mut out: Vec<FallbackRegionPinPlan> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_region_pin_plan(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_fallback_region_pin_plan(item: &FallbackRegionPinPlan) -> String {
    region_pin_draft::region_pin_draft_bucket(item.effective_volume())
}
