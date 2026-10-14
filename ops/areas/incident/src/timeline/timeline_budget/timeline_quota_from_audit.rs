use super::*;

use std::collections::BTreeMap;

use crate::action_item::proposal as action_item_proposal;

pub fn summarize_action_item_audits_for_quota(
    items: &[action_item_proposal::ActionItemAudit],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_timeline_quotas_from_audit(items: &[action_item_proposal::ActionItemAudit]) -> Vec<TimelineQuota> {
    let mut out: Vec<TimelineQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_quota(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_timeline_quota(item: &TimelineQuota) -> String {
    action_item_proposal::classify_action_item_audit(item.projected_value())
}
