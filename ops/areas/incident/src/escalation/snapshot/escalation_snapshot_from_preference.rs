use super::*;

use std::collections::BTreeMap;

use crate::timeline::policy as timeline_policy;

pub fn summarize_lenient_timeline_preferences_for_snapshot(
    items: &[timeline_policy::LenientTimelinePreference],
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

pub fn derive_escalation_snapshots_from_preference(
    items: &[timeline_policy::LenientTimelinePreference],
) -> Vec<EscalationSnapshot> {
    let mut out: Vec<EscalationSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_snapshot(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_escalation_snapshot_extra(item: &EscalationSnapshot) -> f64 {
    timeline_policy::lenient_timeline_preference_weight(7, true)
}
