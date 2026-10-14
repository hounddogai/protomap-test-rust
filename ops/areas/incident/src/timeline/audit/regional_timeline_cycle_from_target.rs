use super::*;

use std::collections::BTreeMap;

use crate::page::budget as page_budget;

pub fn summarize_strict_page_targets_for_cycle(items: &[page_budget::StrictPageTarget]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_regional_timeline_cycles_from_target(
    items: &[page_budget::StrictPageTarget],
) -> Vec<RegionalTimelineCycle> {
    let mut out: Vec<RegionalTimelineCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_timeline_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_regional_timeline_cycle(item: &RegionalTimelineCycle) -> String {
    page_budget::strict_page_target_bucket(item.adjusted_load())
}
