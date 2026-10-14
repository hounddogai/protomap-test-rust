use super::*;

use std::collections::BTreeMap;

use crate::probe::slot as probe_slot;

pub fn summarize_probe_estimates_for_slot(items: &[probe_slot::ProbeEstimate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_legacy_alert_slots_from_estimate(items: &[probe_slot::ProbeEstimate]) -> Vec<LegacyAlertSlot> {
    let mut out: Vec<LegacyAlertSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_alert_slot(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_legacy_alert_slot(item: &LegacyAlertSlot) -> String {
    probe_slot::classify_probe_estimate(item.weighted_total())
}
