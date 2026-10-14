use super::*;

use std::collections::BTreeMap;

use crate::isolation::tier as isolation_tier;

pub fn summarize_legacy_isolation_ledgers_for_notice(
    items: &[isolation_tier::LegacyIsolationLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_region_pin_notices_from_ledger(items: &[isolation_tier::LegacyIsolationLedger]) -> Vec<RegionPinNotice> {
    let mut out: Vec<RegionPinNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_region_pin_notice(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_region_pin_notice(item: &RegionPinNotice) -> String {
    isolation_tier::classify_legacy_isolation_ledger(item.effective_load())
}
