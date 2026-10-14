use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_budget as timeline_timeline_budget;

pub fn summarize_tiered_timeline_notices_for_assignment(
    items: &[timeline_timeline_budget::TieredTimelineNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_default_runbook_assignments_from_notice(
    items: &[timeline_timeline_budget::TieredTimelineNotice],
) -> Vec<DefaultRunbookAssignment> {
    let mut out: Vec<DefaultRunbookAssignment> = Vec::new();
    for item in items.iter() {
        let derived = new_default_runbook_assignment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_default_runbook_assignment(item: &DefaultRunbookAssignment) -> String {
    timeline_timeline_budget::classify_timeline_quota(item.weighted_volume())
}
