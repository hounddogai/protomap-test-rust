use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_bundle as sink_sink_bundle;

pub fn summarize_adaptive_sink_tiers_for_milestone(
    items: &[sink_sink_bundle::AdaptiveSinkTier],
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

pub fn derive_lenient_data_export_milestones_from_tier(
    items: &[sink_sink_bundle::AdaptiveSinkTier],
) -> Vec<LenientDataExportMilestone> {
    let mut out: Vec<LenientDataExportMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_data_export_milestone(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_lenient_data_export_milestone(item: &LenientDataExportMilestone) -> f64 {
    sink_sink_bundle::adaptive_sink_tier_weight(7, true)
}
