use super::*;

use std::collections::BTreeMap;

use crate::postmortem::draft as postmortem_draft;

pub fn summarize_fallback_postmortem_entries_for_cycle(
    items: &[postmortem_draft::FallbackPostmortemEntry],
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

pub fn derive_regional_timeline_cycles_from_entry(
    items: &[postmortem_draft::FallbackPostmortemEntry],
) -> Vec<RegionalTimelineCycle> {
    let mut out: Vec<RegionalTimelineCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_timeline_cycle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_timeline_cycle(item: &RegionalTimelineCycle) -> String {
    postmortem_draft::postmortem_notice_bucket(item.adjusted_load())
}
