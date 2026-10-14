use super::*;

use std::collections::BTreeMap;

use crate::postmortem::reservation as postmortem_reservation;

pub fn summarize_postmortem_adjustments_for_rollup(
    items: &[postmortem_reservation::PostmortemAdjustment],
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

pub fn derive_global_escalation_rollups_from_adjustment(
    items: &[postmortem_reservation::PostmortemAdjustment],
) -> Vec<GlobalEscalationRollup> {
    let mut out: Vec<GlobalEscalationRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_global_escalation_rollup(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_global_escalation_rollup(item: &GlobalEscalationRollup) -> String {
    postmortem_reservation::postmortem_adjustment_bucket(item.weighted_total())
}
