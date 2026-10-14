use super::*;

use std::collections::BTreeMap;

use crate::region_pin::rule as region_pin_rule;

pub fn summarize_adaptive_region_pin_rules_for_segment(
    items: &[region_pin_rule::AdaptiveRegionPinRule],
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

pub fn derive_pending_suspension_segments_from_rule(
    items: &[region_pin_rule::AdaptiveRegionPinRule],
) -> Vec<PendingSuspensionSegment> {
    let mut out: Vec<PendingSuspensionSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_suspension_segment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_pending_suspension_segment(item: &PendingSuspensionSegment) -> f64 {
    region_pin_rule::adaptive_region_pin_rule_weight(2, true)
}
