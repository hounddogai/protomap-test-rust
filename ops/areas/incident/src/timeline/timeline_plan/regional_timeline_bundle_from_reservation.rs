use super::*;

use std::collections::BTreeMap;

use crate::timeline::seasonal_legacy_timeline_policy as timeline_seasonal_legacy_timeline_policy;

pub fn summarize_capped_timeline_reservations_for_bundle(
    items: &[timeline_seasonal_legacy_timeline_policy::CappedTimelineReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_regional_timeline_bundles_from_reservation(
    items: &[timeline_seasonal_legacy_timeline_policy::CappedTimelineReservation],
) -> Vec<RegionalTimelineBundle> {
    let mut out: Vec<RegionalTimelineBundle> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_timeline_bundle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_regional_timeline_bundle(item: &RegionalTimelineBundle) -> String {
    timeline_seasonal_legacy_timeline_policy::classify_capped_timeline_reservation(item.weighted_load())
}
