use super::*;

use std::collections::BTreeMap;

use crate::timeline::timeline_budget as timeline_timeline_budget;

pub fn summarize_tiered_timeline_notices_for_template(
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

pub fn derive_action_item_templates_from_notice(
    items: &[timeline_timeline_budget::TieredTimelineNotice],
) -> Vec<ActionItemTemplate> {
    let mut out: Vec<ActionItemTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_template(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn apply_action_item_template(item: &ActionItemTemplate) -> String {
    timeline_timeline_budget::classify_timeline_quota(item.adjusted_value())
}
