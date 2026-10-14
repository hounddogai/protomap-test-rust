use super::*;

use std::collections::BTreeMap;

use crate::slo::group as slo_group;

pub fn summarize_slo_groups_for_reservation(items: &[slo_group::SloGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_regional_dashboard_reservations_from_group(
    items: &[slo_group::SloGroup],
) -> Vec<RegionalDashboardReservation> {
    let mut out: Vec<RegionalDashboardReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_dashboard_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_regional_dashboard_reservation(item: &RegionalDashboardReservation) -> f64 {
    slo_group::slo_assignment_weight(3, true)
}
