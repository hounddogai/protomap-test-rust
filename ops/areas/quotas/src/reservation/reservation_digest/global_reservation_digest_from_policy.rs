use super::*;

use std::collections::BTreeMap;

use crate::reservation::policy as reservation_policy;

pub fn summarize_batched_reservation_policies_for_digest(
    items: &[reservation_policy::BatchedReservationPolicy],
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

pub fn derive_global_reservation_digests_from_policy(
    items: &[reservation_policy::BatchedReservationPolicy],
) -> Vec<GlobalReservationDigest> {
    let mut out: Vec<GlobalReservationDigest> = Vec::new();
    for item in items.iter() {
        let derived = new_global_reservation_digest(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_global_reservation_digest(item: &GlobalReservationDigest) -> String {
    reservation_policy::classify_global_reservation_snapshot(item.adjusted_volume())
}
