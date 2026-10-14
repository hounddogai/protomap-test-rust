use super::*;

use std::collections::BTreeMap;

use crate::release::release_slot as release_release_slot;

pub fn summarize_tiered_release_slots_for_metric(
    items: &[release_release_slot::TieredReleaseSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_freeze_metrics_from_slot(items: &[release_release_slot::TieredReleaseSlot]) -> Vec<FreezeMetric> {
    let mut out: Vec<FreezeMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_metric(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_freeze_metric(item: &FreezeMetric) -> String {
    release_release_slot::release_review_bucket(item.measured_value())
}
