use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_policy as timeline_timeline_policy;

pub fn summarize_default_timeline_drafts_for_digest(
    items: &[timeline_timeline_policy::DefaultTimelineDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_scheduled_action_item_digests_from_draft(
    items: &[timeline_timeline_policy::DefaultTimelineDraft],
) -> Vec<ScheduledActionItemDigest> {
    let mut out: Vec<ScheduledActionItemDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_action_item_digest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_scheduled_action_item_digest(item: &ScheduledActionItemDigest) -> String {
    timeline_timeline_policy::normalize_primary_timeline_variance_code(&item.code)
}
