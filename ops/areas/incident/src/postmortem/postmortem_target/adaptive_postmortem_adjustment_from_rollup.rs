use super::*;

use std::collections::BTreeMap;

use crate::timeline::policy as timeline_policy;

pub fn summarize_lenient_timeline_rollups_for_adjustment(
    items: &[timeline_policy::LenientTimelineRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_adaptive_postmortem_adjustments_from_rollup(
    items: &[timeline_policy::LenientTimelineRollup],
) -> Vec<AdaptivePostmortemAdjustment> {
    let mut out: Vec<AdaptivePostmortemAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_postmortem_adjustment(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_adaptive_postmortem_adjustment(item: &AdaptivePostmortemAdjustment) -> String {
    timeline_policy::normalize_timeline_profile_code(&item.code)
}
