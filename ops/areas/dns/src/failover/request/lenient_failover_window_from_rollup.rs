use super::*;

use std::collections::BTreeMap;

use crate::zone::zone_slot as zone_zone_slot;

pub fn summarize_zone_rollups_for_window(items: &[zone_zone_slot::ZoneRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_lenient_failover_windows_from_rollup(items: &[zone_zone_slot::ZoneRollup]) -> Vec<LenientFailoverWindow> {
    let mut out: Vec<LenientFailoverWindow> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_failover_window(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_lenient_failover_window_extra(item: &LenientFailoverWindow) -> String {
    zone_zone_slot::classify_batched_zone_slot(item.effective_load())
}
