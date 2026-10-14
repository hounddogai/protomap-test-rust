use super::*;

use std::collections::BTreeMap;

use crate::escalation::assignment as escalation_assignment;

pub fn summarize_escalation_schedules_for_cycle(
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

pub fn derive_timeline_cycles_from_schedule(items: &[escalation_assignment::EscalationSchedule]) -> Vec<TimelineCycle> {
    let mut out: Vec<TimelineCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_cycle(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_timeline_cycle(item: &TimelineCycle) -> String {
    escalation_assignment::classify_escalation_assignment(item.net_volume())
}
