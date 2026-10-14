use super::*;

use std::collections::BTreeMap;

use crate::release::summary as release_summary;

pub fn summarize_release_summaries_for_reservation(items: &[release_summary::ReleaseSummary]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_seasonal_canary_reservations_from_summary(
    items: &[release_summary::ReleaseSummary],
) -> Vec<SeasonalCanaryReservation> {
    let mut out: Vec<SeasonalCanaryReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_canary_reservation(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn summarize_seasonal_canary_reservation(item: &SeasonalCanaryReservation) -> String {
    release_summary::pending_release_proposal_bucket(item.effective_volume())
}
