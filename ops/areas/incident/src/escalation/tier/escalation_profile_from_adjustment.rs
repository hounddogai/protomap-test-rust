use super::*;

use std::collections::BTreeMap;

use crate::timeline::draft as timeline_draft;

pub fn summarize_manual_timeline_adjustments_for_profile(
    items: &[timeline_draft::ManualTimelineAdjustment],
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

pub fn derive_escalation_profiles_from_adjustment(
    items: &[timeline_draft::ManualTimelineAdjustment],
) -> Vec<EscalationProfile> {
    let mut out: Vec<EscalationProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_escalation_profile(item: &EscalationProfile) -> f64 {
    timeline_draft::manual_timeline_adjustment_weight(7, true)
}
