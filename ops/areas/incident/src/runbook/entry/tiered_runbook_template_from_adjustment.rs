use super::*;

use std::collections::BTreeMap;

use crate::timeline::draft as timeline_draft;

pub fn summarize_manual_timeline_adjustments_for_template(
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

pub fn derive_tiered_runbook_templates_from_adjustment(
    items: &[timeline_draft::ManualTimelineAdjustment],
) -> Vec<TieredRunbookTemplate> {
    let mut out: Vec<TieredRunbookTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_runbook_template(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_tiered_runbook_template(item: &TieredRunbookTemplate) -> f64 {
    timeline_draft::manual_timeline_adjustment_weight(6, true)
}
