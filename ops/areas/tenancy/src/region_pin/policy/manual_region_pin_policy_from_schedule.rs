use super::*;

use std::collections::BTreeMap;

use crate::suspension::exception as suspension_exception;

pub fn summarize_scheduled_suspension_schedules_for_policy(
    items: &[suspension_exception::ScheduledSuspensionSchedule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_manual_region_pin_policies_from_schedule(
    items: &[suspension_exception::ScheduledSuspensionSchedule],
) -> Vec<ManualRegionPinPolicy> {
    let mut out: Vec<ManualRegionPinPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_region_pin_policy(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_manual_region_pin_policy(item: &ManualRegionPinPolicy) -> String {
    suspension_exception::normalize_scheduled_suspension_schedule_code(&item.code)
}
