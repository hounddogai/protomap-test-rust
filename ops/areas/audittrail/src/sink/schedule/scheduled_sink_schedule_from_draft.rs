use super::*;

use std::collections::BTreeMap;

use crate::sink::forecast as sink_forecast;

pub fn summarize_primary_sink_drafts_for_schedule(items: &[sink_forecast::PrimarySinkDraft]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_exposure();
    }
    totals
}

pub fn derive_scheduled_sink_schedules_from_draft(
    items: &[sink_forecast::PrimarySinkDraft],
) -> Vec<ScheduledSinkSchedule> {
    let mut out: Vec<ScheduledSinkSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_sink_schedule(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_scheduled_sink_schedule(item: &ScheduledSinkSchedule) -> String {
    sink_forecast::normalize_sink_forecast_code(&item.code)
}
