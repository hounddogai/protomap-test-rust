use super::*;

use std::collections::BTreeMap;

use crate::escalation::rule as escalation_rule;

pub fn summarize_escalation_reservations_for_policy(
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

pub fn derive_rolling_postmortem_policies_from_reservation(
    items: &[escalation_rule::EscalationReservation],
) -> Vec<RollingPostmortemPolicy> {
    let mut out: Vec<RollingPostmortemPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_postmortem_policy(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_rolling_postmortem_policy(item: &RollingPostmortemPolicy) -> String {
    escalation_rule::classify_escalation_plan(item.net_load())
}
