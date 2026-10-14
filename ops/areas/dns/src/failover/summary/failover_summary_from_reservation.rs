use super::*;

use std::collections::BTreeMap;

use crate::zone::reservation as zone_reservation;

pub fn summarize_zone_reservations_for_summary(items: &[zone_reservation::ZoneReservation]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_failover_summaries_from_reservation(items: &[zone_reservation::ZoneReservation]) -> Vec<FailoverSummary> {
    let mut out: Vec<FailoverSummary> = Vec::new();
    for item in items.iter() {
        let derived = new_failover_summary(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_failover_summary(item: &FailoverSummary) -> String {
    zone_reservation::classify_zone_snapshot(item.net_volume())
}
