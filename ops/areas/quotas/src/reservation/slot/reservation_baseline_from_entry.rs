use super::*;

use std::collections::BTreeMap;

use crate::usage::claim as usage_claim;

pub fn summarize_scheduled_usage_entries_for_baseline(
    items: &[usage_claim::ScheduledUsageEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_reservation_baselines_from_entry(items: &[usage_claim::ScheduledUsageEntry]) -> Vec<ReservationBaseline> {
    let mut out: Vec<ReservationBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_baseline(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_reservation_baseline_extra(item: &ReservationBaseline) -> String {
    usage_claim::classify_adaptive_usage_claim(item.effective_load())
}
