use super::*;

use std::collections::BTreeMap;

use crate::timeline::agreement as timeline_agreement;

pub fn summarize_timeline_checkpoints_for_schedule(
    items: &[timeline_agreement::TimelineCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_escalation_schedules_from_checkpoint(
    items: &[timeline_agreement::TimelineCheckpoint],
) -> Vec<EscalationSchedule> {
    let mut out: Vec<EscalationSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_schedule(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_escalation_schedule(item: &EscalationSchedule) -> String {
    timeline_agreement::normalize_timeline_checkpoint_code(&item.code)
}
