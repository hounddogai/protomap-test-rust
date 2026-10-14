use super::*;

use std::collections::BTreeMap;

use crate::failover::summary as failover_summary;

pub fn summarize_manual_failover_baselines_for_segment(
    items: &[failover_summary::ManualFailoverBaseline],
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

pub fn derive_zone_segments_from_baseline(items: &[failover_summary::ManualFailoverBaseline]) -> Vec<ZoneSegment> {
    let mut out: Vec<ZoneSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_segment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_zone_segment(item: &ZoneSegment) -> String {
    failover_summary::classify_failover_summary(item.effective_value())
}
