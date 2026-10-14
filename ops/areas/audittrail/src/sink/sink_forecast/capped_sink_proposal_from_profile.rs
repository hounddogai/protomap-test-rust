use super::*;

use std::collections::BTreeMap;

use crate::audit_event::rollup as audit_event_rollup;

pub fn summarize_pending_audit_event_profiles_for_proposal(
    items: &[audit_event_rollup::PendingAuditEventProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_capped_sink_proposals_from_profile(
    items: &[audit_event_rollup::PendingAuditEventProfile],
) -> Vec<CappedSinkProposal> {
    let mut out: Vec<CappedSinkProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_sink_proposal(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_capped_sink_proposal(item: &CappedSinkProposal) -> String {
    audit_event_rollup::classify_pending_audit_event_profile(item.net_volume())
}
