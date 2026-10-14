use super::*;

use std::collections::BTreeMap;

use crate::probe::record_item as probe_record_item;

pub fn summarize_adaptive_probe_entries_for_slot(
    items: &[probe_record_item::AdaptiveProbeEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_tiered_alert_slots_from_entry(items: &[probe_record_item::AdaptiveProbeEntry]) -> Vec<TieredAlertSlot> {
    let mut out: Vec<TieredAlertSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_alert_slot(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_tiered_alert_slot(item: &TieredAlertSlot) -> String {
    probe_record_item::default_probe_record_bucket(item.projected_load())
}
