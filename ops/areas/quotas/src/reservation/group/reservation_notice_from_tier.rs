use super::*;

use std::collections::BTreeMap;

use crate::grant::reservation as grant_reservation;

pub fn summarize_grant_tiers_for_notice(items: &[grant_reservation::GrantTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_reservation_notices_from_tier(items: &[grant_reservation::GrantTier]) -> Vec<ReservationNotice> {
    let mut out: Vec<ReservationNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_reservation_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_reservation_notice(item: &ReservationNotice) -> f64 {
    grant_reservation::strict_grant_reservation_weight(2, true)
}
