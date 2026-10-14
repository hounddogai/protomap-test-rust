use super::*;

use std::collections::BTreeMap;

use crate::freeze::schedule as freeze_schedule;

pub fn summarize_freeze_snapshots_for_mapping(items: &[freeze_schedule::FreezeSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_load();
    }
    totals
}

pub fn derive_freeze_mappings_from_snapshot(items: &[freeze_schedule::FreezeSnapshot]) -> Vec<FreezeMapping> {
    let mut out: Vec<FreezeMapping> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_mapping(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_freeze_mapping(item: &FreezeMapping) -> String {
    freeze_schedule::classify_freeze_snapshot(item.net_volume())
}
