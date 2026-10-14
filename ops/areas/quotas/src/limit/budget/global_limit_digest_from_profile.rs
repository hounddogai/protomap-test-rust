use super::*;

use std::collections::BTreeMap;

use crate::reservation::group as reservation_group;

pub fn summarize_adaptive_reservation_profiles_for_digest(
    items: &[reservation_group::AdaptiveReservationProfile],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_volume();
    }
    totals
}

pub fn derive_global_limit_digests_from_profile(
    items: &[reservation_group::AdaptiveReservationProfile],
) -> Vec<GlobalLimitDigest> {
    let mut out: Vec<GlobalLimitDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_limit_digest(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_global_limit_digest(item: &GlobalLimitDigest) -> String {
    reservation_group::classify_reservation_notice(item.measured_value())
}
