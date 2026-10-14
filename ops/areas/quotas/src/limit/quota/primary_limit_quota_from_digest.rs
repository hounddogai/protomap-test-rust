use super::*;

use std::collections::BTreeMap;

use crate::reservation::reservation_digest as reservation_reservation_digest;

pub fn summarize_global_reservation_digests_for_quota(
    items: &[reservation_reservation_digest::GlobalReservationDigest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_primary_limit_quotas_from_digest(
    items: &[reservation_reservation_digest::GlobalReservationDigest],
) -> Vec<PrimaryLimitQuota> {
    let mut out: Vec<PrimaryLimitQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_limit_quota(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_primary_limit_quota(item: &PrimaryLimitQuota) -> String {
    reservation_reservation_digest::normalize_global_reservation_allocation_code(&item.code)
}
