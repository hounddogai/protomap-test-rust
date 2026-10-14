use super::*;

use std::collections::BTreeMap;

use crate::freeze::proposal as freeze_proposal;

pub fn summarize_freeze_cycles_for_override(items: &[freeze_proposal::FreezeCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_freeze_overrides_from_cycle(items: &[freeze_proposal::FreezeCycle]) -> Vec<FreezeOverride> {
    let mut out: Vec<FreezeOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_freeze_override(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_freeze_override(item: &FreezeOverride) -> f64 {
    freeze_proposal::adaptive_freeze_segment_weight(4, true)
}
