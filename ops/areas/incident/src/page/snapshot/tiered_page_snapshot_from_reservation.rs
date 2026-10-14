use super::*;

use std::collections::BTreeMap;

use crate::escalation::rule as escalation_rule;

pub fn summarize_escalation_reservations_for_snapshot(
    items: &[escalation_rule::EscalationReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_tiered_page_snapshots_from_reservation(
    items: &[escalation_rule::EscalationReservation],
) -> Vec<TieredPageSnapshot> {
    let mut out: Vec<TieredPageSnapshot> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_page_snapshot(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_tiered_page_snapshot(item: &TieredPageSnapshot) -> String {
    escalation_rule::classify_escalation_plan(item.adjusted_exposure())
}
