use super::*;

use std::collections::BTreeMap;

use crate::escalation::variance as escalation_variance;

pub fn summarize_escalation_manifests_for_adjustment(
    items: &[escalation_variance::EscalationManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_manual_timeline_adjustments_from_manifest(
    items: &[escalation_variance::EscalationManifest],
) -> Vec<ManualTimelineAdjustment> {
    let mut out: Vec<ManualTimelineAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_timeline_adjustment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_manual_timeline_adjustment(item: &ManualTimelineAdjustment) -> String {
    escalation_variance::classify_escalation_manifest(item.weighted_volume())
}
