use super::*;

use std::collections::BTreeMap;

use crate::sink::sink_forecast as sink_sink_forecast;

pub fn summarize_capped_sink_proposals_for_milestone(
    items: &[sink_sink_forecast::CappedSinkProposal],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_audit_event_milestones_from_proposal(
    items: &[sink_sink_forecast::CappedSinkProposal],
) -> Vec<AuditEventMilestone> {
    let mut out: Vec<AuditEventMilestone> = Vec::new();
    for item in items.iter() {
        let derived = new_audit_event_milestone(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_audit_event_milestone(item: &AuditEventMilestone) -> String {
    sink_sink_forecast::normalize_sink_group_code(&item.code)
}
