use super::*;

use std::collections::BTreeMap;

use crate::dashboard::dashboard_ledger as dashboard_dashboard_ledger;

pub fn summarize_lenient_dashboard_ledgers_for_reservation(
    items: &[dashboard_dashboard_ledger::LenientDashboardLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_silence_reservations_from_ledger(
    items: &[dashboard_dashboard_ledger::LenientDashboardLedger],
) -> Vec<SilenceReservation> {
    let mut out: Vec<SilenceReservation> = Vec::new();
    for item in items.iter() {
        let derived = new_silence_reservation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_silence_reservation(item: &SilenceReservation) -> String {
    dashboard_dashboard_ledger::dashboard_record_bucket(item.measured_value())
}
