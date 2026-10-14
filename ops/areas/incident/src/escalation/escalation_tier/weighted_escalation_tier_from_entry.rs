use super::*;

use std::collections::BTreeMap;

use crate::runbook::segment as runbook_segment;

pub fn summarize_adaptive_runbook_entries_for_tier(
    items: &[runbook_segment::AdaptiveRunbookEntry],
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

pub fn derive_weighted_escalation_tiers_from_entry(
    items: &[runbook_segment::AdaptiveRunbookEntry],
) -> Vec<WeightedEscalationTier> {
    let mut out: Vec<WeightedEscalationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_weighted_escalation_tier(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_weighted_escalation_tier(item: &WeightedEscalationTier) -> String {
    runbook_segment::classify_runbook_slot(item.projected_value())
}
