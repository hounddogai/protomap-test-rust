use super::*;

use std::collections::BTreeMap;

use crate::alert::override_item as alert_override_item;

pub fn summarize_alert_overrides_for_reservation(
    items: &[alert_override_item::AlertOverride],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_primary_dashboard_reservations_from_override(
    items: &[alert_override_item::AlertOverride],
) -> Vec<PrimaryDashboardReservation> {
    let mut out: Vec<PrimaryDashboardReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_dashboard_reservation(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_primary_dashboard_reservation(item: &PrimaryDashboardReservation) -> String {
    alert_override_item::classify_alert_request(item.projected_volume())
}
