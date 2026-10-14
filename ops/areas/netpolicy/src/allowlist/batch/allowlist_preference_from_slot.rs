use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_record as peering_peering_record;

pub fn summarize_primary_peering_slots_for_preference(
    items: &[peering_peering_record::PrimaryPeeringSlot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_allowlist_preferences_from_slot(
    items: &[peering_peering_record::PrimaryPeeringSlot],
) -> Vec<AllowlistPreference> {
    let mut out: Vec<AllowlistPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_preference(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_allowlist_preference(item: &AllowlistPreference) -> String {
    peering_peering_record::classify_peering_preference(item.projected_load())
}
