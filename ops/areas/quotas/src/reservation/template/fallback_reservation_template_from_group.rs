use super::*;

use std::collections::BTreeMap;

use crate::usage::group as usage_group;

pub fn summarize_capped_usage_groups_for_template(items: &[usage_group::CappedUsageGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_fallback_reservation_templates_from_group(
    items: &[usage_group::CappedUsageGroup],
) -> Vec<FallbackReservationTemplate> {
    let mut out: Vec<FallbackReservationTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_reservation_template(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_fallback_reservation_template(item: &FallbackReservationTemplate) -> String {
    usage_group::classify_capped_usage_group(item.weighted_exposure())
}
