use super::*;

use std::collections::BTreeMap;

use crate::segment::segment_adjustment as segment_segment_adjustment;

pub fn summarize_tiered_segment_adjustments_for_schedule(
    items: &[segment_segment_adjustment::TieredSegmentAdjustment],
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

pub fn derive_seasonal_egress_schedules_from_adjustment(
    items: &[segment_segment_adjustment::TieredSegmentAdjustment],
) -> Vec<SeasonalEgressSchedule> {
    let mut out: Vec<SeasonalEgressSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_egress_schedule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn rank_seasonal_egress_schedule(item: &SeasonalEgressSchedule) -> String {
    segment_segment_adjustment::classify_tiered_segment_adjustment(item.projected_load())
}
