use super::*;

use std::collections::BTreeMap;

use crate::action_item::assignment as action_item_assignment;

pub fn summarize_fallback_action_item_assignments_for_reservation(
    items: &[action_item_assignment::FallbackActionItemAssignment],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_escalation_reservations_from_assignment(
    items: &[action_item_assignment::FallbackActionItemAssignment],
) -> Vec<EscalationReservation> {
    let mut out: Vec<EscalationReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_escalation_reservation(item: &EscalationReservation) -> String {
    action_item_assignment::classify_fallback_action_item_assignment(item.adjusted_load())
}
