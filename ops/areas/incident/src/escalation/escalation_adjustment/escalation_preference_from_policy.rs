use super::*;

use std::collections::BTreeMap;

use crate::timeline::legacy_timeline_policy as timeline_legacy_timeline_policy;

pub fn summarize_pending_timeline_policies_for_preference(
    items: &[timeline_legacy_timeline_policy::PendingTimelinePolicy],
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

pub fn derive_escalation_preferences_from_policy(
    items: &[timeline_legacy_timeline_policy::PendingTimelinePolicy],
) -> Vec<EscalationPreference> {
    let mut out: Vec<EscalationPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_preference(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_escalation_preference(item: &EscalationPreference) -> String {
    timeline_legacy_timeline_policy::pending_timeline_policy_bucket(item.measured_value())
}
