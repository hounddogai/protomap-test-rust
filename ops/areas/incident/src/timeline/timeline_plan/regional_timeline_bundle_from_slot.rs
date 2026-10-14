use super::*;

use std::collections::BTreeMap;

use crate::escalation::slot as escalation_slot;

pub fn summarize_escalation_slots_for_bundle(items: &[escalation_slot::EscalationSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_regional_timeline_bundles_from_slot(
    items: &[escalation_slot::EscalationSlot],
) -> Vec<RegionalTimelineBundle> {
    let mut out: Vec<RegionalTimelineBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_timeline_bundle(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_regional_timeline_bundle(item: &RegionalTimelineBundle) -> String {
    escalation_slot::primary_escalation_group_bucket(item.weighted_load())
}
