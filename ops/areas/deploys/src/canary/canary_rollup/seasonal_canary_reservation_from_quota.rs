use super::*;

use std::collections::BTreeMap;

use crate::canary::entry as canary_entry;

pub fn summarize_primary_canary_quotas_for_reservation(
    items: &[canary_entry::PrimaryCanaryQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_exposure();
    }
    totals
}

pub fn derive_seasonal_canary_reservations_from_quota(
    items: &[canary_entry::PrimaryCanaryQuota],
) -> Vec<SeasonalCanaryReservation> {
    let mut out: Vec<SeasonalCanaryReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_canary_reservation(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_seasonal_canary_reservation(item: &SeasonalCanaryReservation) -> String {
    canary_entry::primary_canary_quota_bucket(item.effective_volume())
}
