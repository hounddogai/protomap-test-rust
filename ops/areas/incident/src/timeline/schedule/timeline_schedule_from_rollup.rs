use super::*;

use std::collections::BTreeMap;

use crate::escalation::rollup as escalation_rollup;

pub fn summarize_default_escalation_rollups_for_schedule(
    items: &[escalation_rollup::DefaultEscalationRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_timeline_schedules_from_rollup(
    items: &[escalation_rollup::DefaultEscalationRollup],
) -> Vec<TimelineSchedule> {
    let mut out: Vec<TimelineSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_timeline_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_timeline_schedule(item: &TimelineSchedule) -> String {
    escalation_rollup::classify_default_escalation_rollup(item.projected_total())
}
