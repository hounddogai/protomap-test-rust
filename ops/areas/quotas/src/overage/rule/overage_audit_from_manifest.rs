use super::*;

use std::collections::BTreeMap;

use crate::reservation::window as reservation_window;

pub fn summarize_primary_reservation_manifests_for_audit(
    items: &[reservation_window::PrimaryReservationManifest],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_overage_audits_from_manifest(
    items: &[reservation_window::PrimaryReservationManifest],
) -> Vec<OverageAudit> {
    let mut out: Vec<OverageAudit> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_audit(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_overage_audit(item: &OverageAudit) -> String {
    reservation_window::reservation_window_bucket(item.measured_value())
}
