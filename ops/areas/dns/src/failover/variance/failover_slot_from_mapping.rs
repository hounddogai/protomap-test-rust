use super::*;

use std::collections::BTreeMap;

use crate::zone::plan as zone_plan;

pub fn summarize_zone_mappings_for_slot(items: &[zone_plan::ZoneMapping]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_failover_slots_from_mapping(items: &[zone_plan::ZoneMapping]) -> Vec<FailoverSlot> {
    let mut out: Vec<FailoverSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_slot(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_failover_slot(item: &FailoverSlot) -> String {
    zone_plan::normalize_zone_schedule_code(&item.code)
}
