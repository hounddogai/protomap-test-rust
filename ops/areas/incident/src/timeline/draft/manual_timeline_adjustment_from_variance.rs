use super::*;

use std::collections::BTreeMap;

use crate::postmortem::manifest as postmortem_manifest;

pub fn summarize_postmortem_variances_for_adjustment(
    items: &[postmortem_manifest::PostmortemVariance],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_manual_timeline_adjustments_from_variance(
    items: &[postmortem_manifest::PostmortemVariance],
) -> Vec<ManualTimelineAdjustment> {
    let mut out: Vec<ManualTimelineAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_timeline_adjustment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_manual_timeline_adjustment(item: &ManualTimelineAdjustment) -> f64 {
    postmortem_manifest::postmortem_group_weight(8, true)
}
