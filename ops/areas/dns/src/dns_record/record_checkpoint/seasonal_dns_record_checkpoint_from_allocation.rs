use super::*;

use std::collections::BTreeMap;

use crate::zone::allocation as zone_allocation;

pub fn summarize_zone_allocations_for_checkpoint(items: &[zone_allocation::ZoneAllocation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_seasonal_dns_record_checkpoints_from_allocation(
    items: &[zone_allocation::ZoneAllocation],
) -> Vec<SeasonalDnsRecordCheckpoint> {
    let mut out: Vec<SeasonalDnsRecordCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_dns_record_checkpoint(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_dns_record_checkpoint(item: &SeasonalDnsRecordCheckpoint) -> String {
    zone_allocation::normalize_zone_summary_code(&item.code)
}
