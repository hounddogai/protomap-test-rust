use super::*;

use std::collections::BTreeMap;

use crate::retention::reservation as retention_reservation;

pub fn summarize_retention_reservations_for_notice(
    items: &[retention_reservation::RetentionReservation],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_sink_notices_from_reservation(items: &[retention_reservation::RetentionReservation]) -> Vec<SinkNotice> {
    let mut out: Vec<SinkNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_sink_notice(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_sink_notice(item: &SinkNotice) -> String {
    retention_reservation::normalize_seasonal_retention_preference_code(&item.code)
}
