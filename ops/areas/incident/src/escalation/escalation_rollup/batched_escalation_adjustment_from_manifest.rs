use super::*;

use std::collections::BTreeMap;

use crate::timeline::schedule as timeline_schedule;

pub fn summarize_tiered_timeline_manifests_for_adjustment(
    items: &[timeline_schedule::TieredTimelineManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_batched_escalation_adjustments_from_manifest(
    items: &[timeline_schedule::TieredTimelineManifest],
) -> Vec<BatchedEscalationAdjustment> {
    let mut out: Vec<BatchedEscalationAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_batched_escalation_adjustment(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_batched_escalation_adjustment(item: &BatchedEscalationAdjustment) -> f64 {
    timeline_schedule::lenient_timeline_forecast_weight(4, true)
}
