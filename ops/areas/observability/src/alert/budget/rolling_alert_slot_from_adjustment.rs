use super::*;

use std::collections::BTreeMap;

use crate::slo::window as slo_window;

pub fn summarize_slo_adjustments_for_slot(items: &[slo_window::SloAdjustment]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_rolling_alert_slots_from_adjustment(items: &[slo_window::SloAdjustment]) -> Vec<RollingAlertSlot> {
    let mut out: Vec<RollingAlertSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_alert_slot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_rolling_alert_slot(item: &RollingAlertSlot) -> String {
    slo_window::classify_slo_allocation(item.weighted_exposure())
}
