use super::*;

use std::collections::BTreeMap;

use crate::sink::agreement as sink_agreement;

pub fn summarize_sink_slots_for_schedule(items: &[sink_agreement::SinkSlot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_fallback_retention_schedules_from_slot(
    items: &[sink_agreement::SinkSlot],
) -> Vec<FallbackRetentionSchedule> {
    let mut out: Vec<FallbackRetentionSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_retention_schedule(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_fallback_retention_schedule(item: &FallbackRetentionSchedule) -> String {
    sink_agreement::primary_sink_manifest_bucket(item.net_exposure())
}
