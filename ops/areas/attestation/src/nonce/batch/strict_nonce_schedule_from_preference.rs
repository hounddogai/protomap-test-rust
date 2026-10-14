use super::*;

use std::collections::BTreeMap;

use crate::nonce::preference as nonce_preference;

pub fn summarize_regional_nonce_preferences_for_schedule(
    items: &[nonce_preference::RegionalNoncePreference],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_strict_nonce_schedules_from_preference(
    items: &[nonce_preference::RegionalNoncePreference],
) -> Vec<StrictNonceSchedule> {
    let mut out: Vec<StrictNonceSchedule> = Vec::new();
    for item in items.iter() {
        let derived = new_strict_nonce_schedule(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_strict_nonce_schedule(item: &StrictNonceSchedule) -> String {
    nonce_preference::normalize_regional_nonce_preference_code(&item.code)
}
