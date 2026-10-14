use super::*;

use std::collections::BTreeMap;

use crate::escalation::assignment as escalation_assignment;

pub fn summarize_escalation_schedules_for_exception(
    items: &[escalation_assignment::EscalationSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_value();
    }
    totals
}

pub fn derive_action_item_exceptions_from_schedule(
    items: &[escalation_assignment::EscalationSchedule],
) -> Vec<ActionItemException> {
    let mut out: Vec<ActionItemException> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_exception(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_action_item_exception(item: &ActionItemException) -> String {
    escalation_assignment::classify_escalation_assignment(item.weighted_volume())
}
