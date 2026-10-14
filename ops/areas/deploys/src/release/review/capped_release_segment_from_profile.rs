use super::*;

use std::collections::BTreeMap;

use crate::freeze::forecast as freeze_forecast;

pub fn summarize_freeze_profiles_for_segment(items: &[freeze_forecast::FreezeProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_capped_release_segments_from_profile(
    items: &[freeze_forecast::FreezeProfile],
) -> Vec<CappedReleaseSegment> {
    let mut out: Vec<CappedReleaseSegment> = Vec::new();
    for item in items.iter() {
        let derived = new_capped_release_segment(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_capped_release_segment_extra(item: &CappedReleaseSegment) -> f64 {
    freeze_forecast::freeze_profile_weight(8, true)
}
