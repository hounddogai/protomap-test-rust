use super::*;

use std::collections::BTreeMap;

use crate::alert::reservation as alert_reservation;

pub fn summarize_alert_bundles_for_preference(items: &[alert_reservation::AlertBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_strict_alert_preferences_from_bundle(
    items: &[alert_reservation::AlertBundle],
) -> Vec<StrictAlertPreference> {
    let mut out: Vec<StrictAlertPreference> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_alert_preference(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_strict_alert_preference(item: &StrictAlertPreference) -> String {
    alert_reservation::normalize_alert_reservation_code(&item.code)
}
