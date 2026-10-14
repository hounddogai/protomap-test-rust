use super::*;

use std::collections::BTreeMap;

use crate::segment::forecast as segment_forecast;

pub fn summarize_pending_segment_preferences_for_digest(
    items: &[segment_forecast::PendingSegmentPreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_scheduled_allowlist_digests_from_preference(
    items: &[segment_forecast::PendingSegmentPreference],
) -> Vec<ScheduledAllowlistDigest> {
    let mut out: Vec<ScheduledAllowlistDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_allowlist_digest(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn apply_scheduled_allowlist_digest(item: &ScheduledAllowlistDigest) -> String {
    segment_forecast::classify_tiered_segment_forecast(item.weighted_load())
}
